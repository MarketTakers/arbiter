use std::sync::Mutex;

use alloy::primitives::Address;
use chacha20poly1305::{AeadInPlace, XChaCha20Poly1305, XNonce, aead::KeyInit};
use diesel::{ExpressionMethods as _, QueryDsl as _, SelectableHelper, dsl::update};
use diesel_async::{AsyncConnection, RunQueryDsl};
use kameo::error::SendError;
use kameo::messages;
use kameo::prelude::Context;
use tracing::{error, info};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::actors::flow_coordinator::client_connect_approval::ClientApprovalAnswer;
use crate::actors::keyholder::KeyHolderState;
use crate::actors::user_agent::session::Error;
use crate::crypto::integrity::v1::USERAGENT_INTEGRITY_TAG;
use crate::db::models::{
    EvmWalletAccess, KeyType, NewEvmWalletAccess, ProgramClient, ProgramClientMetadata,
};
use crate::evm::policies::{Grant, SpecificGrant};
use crate::safe_cell::SafeCell;
use crate::{
    actors::{
        evm::{
            Generate, ListWallets, UseragentCreateGrant, UseragentDeleteGrant, UseragentListGrants,
        },
        keyholder::{self, Bootstrap, SignIntegrityTag, TryUnseal},
        user_agent::session::{
            UserAgentSession,
            state::{UnsealContext, UserAgentEvents, UserAgentStates},
        },
    },
    safe_cell::SafeCellHandle as _,
};

impl UserAgentSession {
    fn take_unseal_secret(&mut self) -> Result<(EphemeralSecret, PublicKey), Error> {
        let UserAgentStates::WaitingForUnsealKey(unseal_context) = self.state.state() else {
            error!("Received encrypted key in invalid state");
            return Err(Error::internal("Invalid state for unseal encrypted key"));
        };

        let ephemeral_secret = {
            #[allow(
                clippy::unwrap_used,
                reason = "Mutex poison is unrecoverable and should panic"
            )]
            let mut secret_lock = unseal_context.secret.lock().unwrap();
            let secret = secret_lock.take();
            match secret {
                Some(secret) => secret,
                None => {
                    drop(secret_lock);
                    error!("Ephemeral secret already taken");
                    return Err(Error::internal("Ephemeral secret already taken"));
                }
            }
        };

        Ok((ephemeral_secret, unseal_context.client_public_key))
    }

    fn decrypt_client_key_material(
        ephemeral_secret: EphemeralSecret,
        client_public_key: PublicKey,
        nonce: &[u8],
        ciphertext: &[u8],
        associated_data: &[u8],
    ) -> Result<SafeCell<Vec<u8>>, ()> {
        let nonce = XNonce::from_slice(nonce);

        let shared_secret = ephemeral_secret.diffie_hellman(&client_public_key);
        let cipher = XChaCha20Poly1305::new(shared_secret.as_bytes().into());

        let mut key_buffer = SafeCell::new(ciphertext.to_vec());

        let decryption_result = key_buffer.write_inline(|write_handle| {
            cipher.decrypt_in_place(nonce, associated_data, write_handle)
        });

        match decryption_result {
            Ok(_) => Ok(key_buffer),
            Err(err) => {
                error!(?err, "Failed to decrypt encrypted key material");
                Err(())
            }
        }
    }

    async fn backfill_missing_useragent_pubkey_integrity_tags(&mut self) -> Result<(), Error> {
        use crate::db::schema::useragent_client;

        let mut conn = self.props.db.get().await?;
        let missing_rows: Vec<(i32, Vec<u8>, KeyType)> = useragent_client::table
            .filter(useragent_client::pubkey_integrity_tag.is_null())
            .select((
                useragent_client::id,
                useragent_client::public_key,
                useragent_client::key_type,
            ))
            .load(&mut conn)
            .await?;
        drop(conn);

        if missing_rows.is_empty() {
            return Ok(());
        }

        let mut updates = Vec::with_capacity(missing_rows.len());
        for (id, public_key, key_type) in missing_rows {
            let tag = self
                .props
                .actors
                .key_holder
                .ask(SignIntegrityTag {
                    purpose_tag: USERAGENT_INTEGRITY_TAG.to_vec(),
                    data_parts: vec![(key_type as i32).to_be_bytes().to_vec(), public_key],
                })
                .await
                .map_err(|err| {
                    error!(?err, "Failed to sign integrity tag");
                    Error::internal("Failed to sign integrity tag")
                })?;
            updates.push((id, tag));
        }

        let mut conn = self.props.db.get().await?;
        for (id, tag) in updates {
            update(useragent_client::table)
                .filter(useragent_client::id.eq(id))
                .set(useragent_client::pubkey_integrity_tag.eq(Some(tag)))
                .execute(&mut conn)
                .await?;
        }

        info!("Backfilled missing user-agent pubkey integrity tags");
        Ok(())
    }
}

pub struct UnsealStartResponse {
    pub server_pubkey: PublicKey,
}

#[derive(Debug, Error)]
pub enum UnsealError {
    #[error("Invalid key provided for unsealing")]
    InvalidKey,
    #[error("Internal error during unsealing process")]
    General(#[from] super::Error),
}

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("Invalid key provided for bootstrapping")]
    InvalidKey,
    #[error("Vault is already bootstrapped")]
    AlreadyBootstrapped,

    #[error("Internal error during bootstrapping process")]
    General(#[from] super::Error),
}

#[messages]
impl UserAgentSession {
    #[message]
    pub async fn handle_unseal_request(
        &mut self,
        client_pubkey: x25519_dalek::PublicKey,
    ) -> Result<UnsealStartResponse, Error> {
        let secret = EphemeralSecret::random();
        let public_key = PublicKey::from(&secret);

        self.transition(UserAgentEvents::UnsealRequest(UnsealContext {
            secret: Mutex::new(Some(secret)),
            client_public_key: client_pubkey,
        }))?;

        Ok(UnsealStartResponse {
            server_pubkey: public_key,
        })
    }

    #[message]
    pub async fn handle_unseal_encrypted_key(
        &mut self,
        nonce: Vec<u8>,
        ciphertext: Vec<u8>,
        associated_data: Vec<u8>,
    ) -> Result<(), UnsealError> {
        let (ephemeral_secret, client_public_key) = match self.take_unseal_secret() {
            Ok(values) => values,
            Err(Error::State) => {
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                return Err(UnsealError::InvalidKey);
            }
            Err(_err) => {
                return Err(Error::internal("Failed to take unseal secret").into());
            }
        };

        let seal_key_buffer = match Self::decrypt_client_key_material(
            ephemeral_secret,
            client_public_key,
            &nonce,
            &ciphertext,
            &associated_data,
        ) {
            Ok(buffer) => buffer,
            Err(()) => {
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                return Err(UnsealError::InvalidKey);
            }
        };

        match self
            .props
            .actors
            .key_holder
            .ask(TryUnseal {
                seal_key_raw: seal_key_buffer,
            })
            .await
        {
            Ok(_) => {
                self.backfill_missing_useragent_pubkey_integrity_tags()
                    .await?;
                info!("Successfully unsealed key with client-provided key");
                self.transition(UserAgentEvents::ReceivedValidKey)?;
                Ok(())
            }
            Err(SendError::HandlerError(keyholder::Error::InvalidKey)) => {
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                Err(UnsealError::InvalidKey)
            }
            Err(SendError::HandlerError(err)) => {
                error!(?err, "Keyholder failed to unseal key");
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                Err(UnsealError::InvalidKey)
            }
            Err(err) => {
                error!(?err, "Failed to send unseal request to keyholder");
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                Err(Error::internal("Vault actor error").into())
            }
        }
    }

    #[message]
    pub(crate) async fn handle_bootstrap_encrypted_key(
        &mut self,
        nonce: Vec<u8>,
        ciphertext: Vec<u8>,
        associated_data: Vec<u8>,
    ) -> Result<(), BootstrapError> {
        let (ephemeral_secret, client_public_key) = match self.take_unseal_secret() {
            Ok(values) => values,
            Err(Error::State) => {
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                return Err(BootstrapError::InvalidKey);
            }
            Err(err) => return Err(err.into()),
        };

        let seal_key_buffer = match Self::decrypt_client_key_material(
            ephemeral_secret,
            client_public_key,
            &nonce,
            &ciphertext,
            &associated_data,
        ) {
            Ok(buffer) => buffer,
            Err(()) => {
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                return Err(BootstrapError::InvalidKey);
            }
        };

        match self
            .props
            .actors
            .key_holder
            .ask(Bootstrap {
                seal_key_raw: seal_key_buffer,
            })
            .await
        {
            Ok(_) => {
                self.backfill_missing_useragent_pubkey_integrity_tags()
                    .await?;
                info!("Successfully bootstrapped vault with client-provided key");
                self.transition(UserAgentEvents::ReceivedValidKey)?;
                Ok(())
            }
            Err(SendError::HandlerError(keyholder::Error::AlreadyBootstrapped)) => {
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                Err(BootstrapError::AlreadyBootstrapped)
            }
            Err(SendError::HandlerError(err)) => {
                error!(?err, "Keyholder failed to bootstrap vault");
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                Err(BootstrapError::InvalidKey)
            }
            Err(err) => {
                error!(?err, "Failed to send bootstrap request to keyholder");
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                Err(BootstrapError::General(Error::internal(
                    "Vault actor error",
                )))
            }
        }
    }
}

#[messages]
impl UserAgentSession {
    #[message]
    pub(crate) async fn handle_query_vault_state(&mut self) -> Result<KeyHolderState, Error> {
        use crate::actors::keyholder::GetState;

        let vault_state = match self.props.actors.key_holder.ask(GetState {}).await {
            Ok(state) => state,
            Err(err) => {
                error!(?err, actor = "useragent", "keyholder.query.failed");
                return Err(Error::internal("Vault is in broken state"));
            }
        };

        Ok(vault_state)
    }
}

#[messages]
impl UserAgentSession {
    #[message]
    pub(crate) async fn handle_evm_wallet_create(&mut self) -> Result<(i32, Address), Error> {
        match self.props.actors.evm.ask(Generate {}).await {
            Ok(address) => Ok(address),
            Err(SendError::HandlerError(err)) => Err(Error::internal(format!(
                "EVM wallet generation failed: {err}"
            ))),
            Err(err) => {
                error!(?err, "EVM actor unreachable during wallet create");
                Err(Error::internal("EVM actor unreachable"))
            }
        }
    }

    #[message]
    pub(crate) async fn handle_evm_wallet_list(&mut self) -> Result<Vec<(i32, Address)>, Error> {
        match self.props.actors.evm.ask(ListWallets {}).await {
            Ok(wallets) => Ok(wallets),
            Err(err) => {
                error!(?err, "EVM wallet list failed");
                Err(Error::internal("Failed to list EVM wallets"))
            }
        }
    }
}

#[messages]
impl UserAgentSession {
    #[message]
    pub(crate) async fn handle_grant_list(&mut self) -> Result<Vec<Grant<SpecificGrant>>, Error> {
        match self.props.actors.evm.ask(UseragentListGrants {}).await {
            Ok(grants) => Ok(grants),
            Err(err) => {
                error!(?err, "EVM grant list failed");
                Err(Error::internal("Failed to list EVM grants"))
            }
        }
    }

    #[message]
    pub(crate) async fn handle_grant_create(
        &mut self,
        basic: crate::evm::policies::SharedGrantSettings,
        grant: crate::evm::policies::SpecificGrant,
    ) -> Result<i32, Error> {
        match self
            .props
            .actors
            .evm
            .ask(UseragentCreateGrant { basic, grant })
            .await
        {
            Ok(grant_id) => Ok(grant_id),
            Err(err) => {
                error!(?err, "EVM grant create failed");
                Err(Error::internal("Failed to create EVM grant"))
            }
        }
    }

    #[message]
    pub(crate) async fn handle_grant_delete(&mut self, grant_id: i32) -> Result<(), Error> {
        match self
            .props
            .actors
            .evm
            .ask(UseragentDeleteGrant { grant_id })
            .await
        {
            Ok(()) => Ok(()),
            Err(err) => {
                error!(?err, "EVM grant delete failed");
                Err(Error::internal("Failed to delete EVM grant"))
            }
        }
    }

    #[message]
    pub(crate) async fn handle_grant_evm_wallet_access(
        &mut self,
        entries: Vec<NewEvmWalletAccess>,
    ) -> Result<(), Error> {
        let mut conn = self.props.db.get().await?;
        conn.transaction(|conn| {
            Box::pin(async move {
                use crate::db::schema::evm_wallet_access;

                for entry in entries {
                    diesel::insert_into(evm_wallet_access::table)
                        .values(&entry)
                        .on_conflict_do_nothing()
                        .execute(conn)
                        .await?;
                }

                Result::<_, Error>::Ok(())
            })
        })
        .await?;
        Ok(())
    }

    #[message]
    pub(crate) async fn handle_revoke_evm_wallet_access(
        &mut self,
        entries: Vec<i32>,
    ) -> Result<(), Error> {
        let mut conn = self.props.db.get().await?;
        conn.transaction(|conn| {
            Box::pin(async move {
                use crate::db::schema::evm_wallet_access;
                for entry in entries {
                    diesel::delete(evm_wallet_access::table)
                        .filter(evm_wallet_access::wallet_id.eq(entry))
                        .execute(conn)
                        .await?;
                }

                Result::<_, Error>::Ok(())
            })
        })
        .await?;
        Ok(())
    }

    #[message]
    pub(crate) async fn handle_list_wallet_access(
        &mut self,
    ) -> Result<Vec<EvmWalletAccess>, Error> {
        let mut conn = self.props.db.get().await?;
        use crate::db::schema::evm_wallet_access;
        let access_entries = evm_wallet_access::table
            .select(EvmWalletAccess::as_select())
            .load::<_>(&mut conn)
            .await?;
        Ok(access_entries)
    }
}

#[messages]
impl UserAgentSession {
    #[message(ctx)]
    pub(crate) async fn handle_new_client_approve(
        &mut self,
        approved: bool,
        pubkey: ed25519_dalek::VerifyingKey,
        ctx: &mut Context<Self, Result<(), Error>>,
    ) -> Result<(), Error> {
        let pending_approval = match self.pending_client_approvals.remove(&pubkey) {
            Some(approval) => approval,
            None => {
                error!("Received client connection response for unknown client");
                return Err(Error::internal("Unknown client in connection response"));
            }
        };

        pending_approval
            .controller
            .tell(ClientApprovalAnswer { approved })
            .await
            .map_err(|err| {
                error!(
                    ?err,
                    "Failed to send client approval response to controller"
                );
                Error::internal("Failed to send client approval response to controller")
            })?;

        ctx.actor_ref().unlink(&pending_approval.controller).await;

        Ok(())
    }

    #[message]
    pub(crate) async fn handle_sdk_client_list(
        &mut self,
    ) -> Result<Vec<(ProgramClient, ProgramClientMetadata)>, Error> {
        use crate::db::schema::{client_metadata, program_client};
        let mut conn = self.props.db.get().await?;

        let clients = program_client::table
            .inner_join(client_metadata::table)
            .select((
                ProgramClient::as_select(),
                ProgramClientMetadata::as_select(),
            ))
            .load::<(ProgramClient, ProgramClientMetadata)>(&mut conn)
            .await?;

        Ok(clients)
    }
}
