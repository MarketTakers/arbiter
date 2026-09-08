use super::AuthenticatedOperator;
use crate::{
    actors::{
        GlobalActors,
        vault::{self, Bootstrap, GetState, TryUnseal, VaultState, events},
        vault_coordinator::{
            ContributeBootstrap, ContributeRecoveryBootstrap, ContributeRecoveryUnseal,
            ContributeUnseal, StartBootstrap,
        },
    },
    crypto::{KeyCell, integrity::{self}},
    db::DatabasePool,
};
use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
use state::State;

use chacha20poly1305::{AeadInPlace, KeyInit as _, XChaCha20Poly1305, XNonce};
use kameo::{Actor, error::SendError, messages, prelude::Message};
use kameo_actors::message_bus::Register;
use tokio::sync::oneshot;
use tracing::{error, info};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

pub use VaultGateMessage as Inbound;
pub use VaultGateMessageReply as Outbound;

pub mod state;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Vault is already bootstrapped")]
    AlreadyBootstrapped,
    #[error("Invalid key provided")]
    InvalidKey,

    #[error("State transition failed")]
    State,

    /// §3.5: ordinary and recovery operators hold different shares of the same split, so each
    /// contribution belongs to exactly one of the two roles. A refusal here is a policy answer
    /// and is kept out of `Internal`, which carries genuine faults.
    #[error("This operator role may not perform that vault action")]
    RoleNotPermitted,

    #[error("Internal error: {0}")]
    Internal(String),
}
impl Error {
    fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

pub struct HandshakeResponse {
    pub server_pubkey: PublicKey,
}

pub struct VaultGate {
    pub auth_creds: AuthenticatedOperator,
    pub promotion_tx: Option<oneshot::Sender<Result<(), Error>>>,
    pub state: State,
    pub actors: GlobalActors,
    pub db: DatabasePool,
}

impl VaultGate {
    pub fn new(
        auth_creds: AuthenticatedOperator,
        actors: GlobalActors,
        db: DatabasePool,
        promotion_tx: oneshot::Sender<Result<(), Error>>,
    ) -> Self {
        Self {
            auth_creds,
            state: State::default(),
            actors,
            db,
            promotion_tx: Some(promotion_tx),
        }
    }
}

impl Actor for VaultGate {
    type Args = Self;

    type Error = ();

    async fn on_start(
        args: Self::Args,
        actor_ref: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let _ = args
            .actors
            .events
            .tell(Register(
                actor_ref.clone().recipient::<events::Bootstrapped>(),
            ))
            .await;
        let _ = args
            .actors
            .events
            .tell(Register(actor_ref.recipient::<events::Unsealed>()))
            .await;
        Ok(args)
    }
}

impl VaultGate {
    /// The id of the ordinary operator on the other end, or a refusal.
    ///
    /// The id is read from the handshake rather than from the request body, so a peer cannot
    /// name an operator it did not authenticate as.
    const fn ordinary_id(&self) -> Result<i32, Error> {
        match &self.auth_creds {
            AuthenticatedOperator::Ordinary(credentials) => Ok(credentials.id),
            AuthenticatedOperator::Recovery(_) => Err(Error::RoleNotPermitted),
        }
    }

    /// The id of the recovery operator on the other end, or a refusal. See `ordinary_id`.
    const fn recovery_id(&self) -> Result<i32, Error> {
        match &self.auth_creds {
            AuthenticatedOperator::Recovery(credentials) => Ok(credentials.id),
            AuthenticatedOperator::Ordinary(_) => Err(Error::RoleNotPermitted),
        }
    }

    fn decrypt_key(
        secret: &SharedSecret,
        nonce: &[u8],
        ciphertext: &[u8],
        associated_data: &[u8],
    ) -> Result<KeyCell, ()> {
        let nonce = XNonce::from_slice(nonce);
        let cipher = XChaCha20Poly1305::new(secret.as_bytes().into());
        let mut key_buffer = SafeCell::new(ciphertext.to_vec());

        let decryption_result = key_buffer.write_inline(|write_handle| {
            cipher.decrypt_in_place(nonce, associated_data, write_handle)
        });

        match decryption_result {
            Ok(()) => KeyCell::try_from(key_buffer).map_err(|()| {
                error!("Decrypted key material has unexpected length");
            }),
            Err(err) => {
                error!(?err, "Failed to decrypt encrypted key material");
                Err(())
            }
        }
    }
}

#[messages(enum)]
impl VaultGate {
    #[message]
    pub fn handle_handshake(
        &mut self,
        client_pubkey: PublicKey,
    ) -> Result<HandshakeResponse, Error> {
        let ephemeral_secret = EphemeralSecret::random();
        let public_key = PublicKey::from(&ephemeral_secret);

        let secret = ephemeral_secret.diffie_hellman(&client_pubkey);

        self.state = State::ReadyForExchange {
            server_key: public_key,
            secret,
        };

        Ok(HandshakeResponse {
            server_pubkey: public_key,
        })
    }

    /// Deliberately open to both roles, unlike `handle_bootstrap_encrypted_key` below.
    ///
    /// Handing over the whole seal key to open a sealed vault is participating in unsealing,
    /// which §3.5 grants a recovery operator, and the peer has to hold that key already -- it
    /// gains nothing here it did not bring. Bootstrap is the opposite: it *chooses* the key for
    /// a vault that has none, which is sole custody of the root key and belongs to no §3.5
    /// power. The reasoning that admits one does not admit the other.
    #[message]
    pub async fn handle_unseal_encrypted_key(
        &mut self,
        nonce: Vec<u8>,
        ciphertext: Vec<u8>,
        associated_data: Vec<u8>,
    ) -> Result<(), Error> {
        let State::ReadyForExchange { secret, .. } = &self.state else {
            return Err(Error::State);
        };

        let Ok(seal_key) = Self::decrypt_key(secret, &nonce, &ciphertext, &associated_data) else {
            return Err(Error::InvalidKey);
        };

        match self
            .actors
            .vault
            .ask(TryUnseal { seal_key })
            .await
        {
            Ok(()) => {
                info!("Successfully unsealed key with client-provided key");
                Ok(())
            }
            Err(SendError::HandlerError(vault::Error::InvalidKey)) => Err(Error::InvalidKey),
            Err(SendError::HandlerError(err)) => {
                error!(?err, "Vault failed to unseal key");
                Err(Error::InvalidKey)
            }
            Err(err) => {
                error!(?err, "Failed to send unseal request to vault");
                Err(Error::internal("Vault actor error"))
            }
        }
    }

    /// §3.4/§3.5: bootstrapping picks the root key for a vault that has none, so whoever gets
    /// here holds sole custody until the committee splits it. That is not one of a recovery
    /// operator's two powers, and the check comes first because the vault commits before this
    /// handler could refuse anything afterwards.
    #[message]
    pub async fn handle_bootstrap_encrypted_key(
        &mut self,
        nonce: Vec<u8>,
        ciphertext: Vec<u8>,
        associated_data: Vec<u8>,
    ) -> Result<(), Error> {
        let _ = self.ordinary_id()?;

        let State::ReadyForExchange { secret, .. } = &self.state else {
            return Err(Error::State);
        };

        let Ok(seal_key) = Self::decrypt_key(secret, &nonce, &ciphertext, &associated_data) else {
            return Err(Error::InvalidKey);
        };

        match self
            .actors
            .vault
            .ask(Bootstrap { seal_key })
            .await
        {
            Ok(()) => {
                info!("Successfully bootstrapped vault with client-provided key");
                Ok(())
            }
            Err(SendError::HandlerError(vault::Error::AlreadyBootstrapped)) => {
                Err(Error::AlreadyBootstrapped)
            }
            Err(SendError::HandlerError(err)) => {
                error!(?err, "Vault failed to bootstrap vault");
                Err(Error::InvalidKey)
            }
            Err(err) => {
                error!(?err, "Failed to send bootstrap request to vault");
                Err(Error::internal("Vault error"))
            }
        }
    }

    #[message]
    pub async fn handle_vault_state(&mut self) -> Result<VaultState, Error> {
        let answer = self
            .actors
            .vault
            .ask(GetState {})
            .await
            .map_err(|_| Error::internal("failed to query vault"))?;

        Ok(answer)
    }

    #[message]
    pub async fn handle_declare_committee(
        &mut self,
        count: usize,
        recovery_count: usize,
    ) -> Result<(), Error> {
        let operator_id = self.ordinary_id()?;

        self.actors
            .vault_coordinator
            .ask(StartBootstrap {
                operator_id,
                declared_count: count,
                recovery_count,
            })
            .await
            .map_err(|_| Error::internal("VaultCoordinator unavailable"))
    }

    #[message]
    pub async fn handle_contribute_bootstrap_passphrase(
        &mut self,
        passphrase: Vec<u8>,
    ) -> Result<bool, Error> {
        let operator_id = self.ordinary_id()?;

        let passphrase_cell = SafeCell::new(passphrase);
        self.actors
            .vault_coordinator
            .ask(ContributeBootstrap {
                operator_id,
                passphrase: passphrase_cell,
            })
            .await
            .map_err(|_| Error::internal("VaultCoordinator unavailable"))
    }

    #[message]
    pub async fn handle_contribute_recovery_bootstrap_passphrase(
        &mut self,
        passphrase: Vec<u8>,
    ) -> Result<bool, Error> {
        let recovery_operator_id = self.recovery_id()?;

        let passphrase_cell = SafeCell::new(passphrase);
        self.actors
            .vault_coordinator
            .ask(ContributeRecoveryBootstrap {
                recovery_operator_id,
                passphrase: passphrase_cell,
            })
            .await
            .map_err(|_| Error::internal("VaultCoordinator unavailable"))
    }

    #[message]
    pub async fn handle_contribute_unseal_passphrase(
        &mut self,
        passphrase: Vec<u8>,
    ) -> Result<bool, Error> {
        let operator_id = self.ordinary_id()?;

        let passphrase_cell = SafeCell::new(passphrase);
        self.actors
            .vault_coordinator
            .ask(ContributeUnseal {
                operator_id,
                passphrase: passphrase_cell,
            })
            .await
            .map_err(|_| Error::internal("VaultCoordinator unavailable"))
    }

    #[message]
    pub async fn handle_contribute_recovery_unseal_passphrase(
        &mut self,
        passphrase: Vec<u8>,
    ) -> Result<bool, Error> {
        let recovery_operator_id = self.recovery_id()?;

        let passphrase_cell = SafeCell::new(passphrase);
        self.actors
            .vault_coordinator
            .ask(ContributeRecoveryUnseal {
                recovery_operator_id,
                passphrase: passphrase_cell,
            })
            .await
            .map_err(|_| Error::internal("VaultCoordinator unavailable"))
    }
}

impl Message<events::Bootstrapped> for VaultGate {
    type Reply = ();

    async fn handle(
        &mut self,
        _: events::Bootstrapped,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let result = async {
            let mut conn = self
                .db
                .get()
                .await
                .map_err(|_| Error::internal("DB unavailable"))?;
            // Each role signs under its own `Integrable::KIND`, so the two id spaces cannot
            // collide in `integrity_envelope`.
            match &self.auth_creds {
                AuthenticatedOperator::Ordinary(credentials) => {
                    integrity::sign_entity(
                        &mut conn,
                        &self.actors.vault,
                        credentials,
                        credentials.id,
                    )
                    .await
                }
                AuthenticatedOperator::Recovery(credentials) => {
                    integrity::sign_entity(
                        &mut conn,
                        &self.actors.vault,
                        credentials,
                        credentials.id,
                    )
                    .await
                }
            }
            .map_err(|e| {
                error!(?e, "Failed to sign integrity envelope on bootstrap");
                Error::internal("Integrity sign failed")
            })?;
            Ok(())
        }
        .await;

        if let Some(tx) = self.promotion_tx.take() {
            let _ = tx.send(result);
        }
        ctx.stop();
    }
}

impl Message<events::Unsealed> for VaultGate {
    type Reply = ();

    async fn handle(
        &mut self,
        _: events::Unsealed,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let Some(tx) = self.promotion_tx.take() {
            let _ = tx.send(Ok(()));
        }
        ctx.stop();
    }
}
