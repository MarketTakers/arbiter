use super::Credentials;
use crate::{
    actors::{
        GlobalActors,
        vault::{self, Bootstrap, GetState, TryUnseal, VaultState, events},
    },
    crypto::integrity::{self},
    db::DatabasePool,
};
use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
use state::State;

use chacha20poly1305::{AeadInOut, KeyInit as _, XChaCha20Poly1305, XNonce};
use kameo::{Actor, error::SendError, messages, prelude::Message};
use kameo_actors::message_bus::Register;
use tokio::sync::oneshot;
use tracing::{error, info};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

pub mod state;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Vault is already bootstrapped")]
    AlreadyBootstrapped,
    #[error("Invalid key provided")]
    InvalidKey,
    #[error("Vault locked: too many failed unseal attempts")]
    LockedOut,

    #[error("State transition failed")]
    State,

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
    pub auth_creds: Credentials,
    pub promotion_tx: Option<oneshot::Sender<Result<(), Error>>>,
    pub state: State,
    pub actors: GlobalActors,
    pub db: DatabasePool,
}

impl VaultGate {
    pub fn new(
        auth_creds: Credentials,
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
    fn decrypt_key(
        secret: &SharedSecret,
        nonce: &[u8],
        ciphertext: &[u8],
        associated_data: &[u8],
    ) -> Result<SafeCell<Vec<u8>>, ()> {
        let Ok(nonce) = XNonce::try_from(nonce) else {
            error!("Encrypted key material carries a nonce of the wrong length");
            return Err(());
        };

        let cipher = XChaCha20Poly1305::new(secret.as_bytes().into());

        let mut key_buffer = SafeCell::new(ciphertext.to_vec());

        let decryption_result = key_buffer.write_inline(|write_handle| {
            cipher.decrypt_in_place(&nonce, associated_data, write_handle)
        });

        match decryption_result {
            Ok(()) => Ok(key_buffer),
            Err(err) => {
                error!(?err, "Failed to decrypt encrypted key material");
                Err(())
            }
        }
    }
}

#[messages(messages = Inbound, replies = Outbound)]
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

        let Ok(seal_key_buffer) = Self::decrypt_key(secret, &nonce, &ciphertext, &associated_data)
        else {
            return Err(Error::InvalidKey);
        };

        match self
            .actors
            .vault
            .ask(TryUnseal {
                seal_key_raw: seal_key_buffer,
            })
            .await
        {
            Ok(()) => {
                info!("Successfully unsealed key with client-provided key");
                Ok(())
            }
            Err(SendError::HandlerError(vault::Error::InvalidKey)) => Err(Error::InvalidKey),
            Err(SendError::HandlerError(vault::Error::LockedOut)) => Err(Error::LockedOut),
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

    #[message]
    pub async fn handle_bootstrap_encrypted_key(
        &mut self,
        nonce: Vec<u8>,
        ciphertext: Vec<u8>,
        associated_data: Vec<u8>,
    ) -> Result<(), Error> {
        let State::ReadyForExchange { secret, .. } = &self.state else {
            return Err(Error::State);
        };

        let Ok(seal_key_buffer) = Self::decrypt_key(secret, &nonce, &ciphertext, &associated_data)
        else {
            return Err(Error::InvalidKey);
        };

        match self
            .actors
            .vault
            .ask(Bootstrap {
                seal_key_raw: seal_key_buffer,
            })
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
            integrity::sign_entity(
                &mut conn,
                &self.actors.vault,
                &self.auth_creds,
                self.auth_creds.id,
            )
            .await
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
