use std::{ops::DerefMut, sync::Mutex};

use chacha20poly1305::{AeadInPlace, XChaCha20Poly1305, XNonce, aead::KeyInit};
use kameo::error::SendError;
use memsafe::MemSafe;
use tracing::{error, info};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::actors::{
    evm::{Generate, ListWallets},
    keyholder::{self, Bootstrap, TryUnseal},
    user_agent::{
        BootstrapError, Request, Response, TransportResponseError, UnsealError, VaultState,
        session::{
            UserAgentSession,
            state::{UnsealContext, UserAgentEvents, UserAgentStates},
        },
    },
};

impl UserAgentSession {
    pub async fn process_transport_inbound(&mut self, req: Request) -> Output {
        match req {
            Request::UnsealStart { client_pubkey } => {
                self.handle_unseal_request(client_pubkey).await
            }
            Request::UnsealEncryptedKey {
                nonce,
                ciphertext,
                associated_data,
            } => {
                self.handle_unseal_encrypted_key(nonce, ciphertext, associated_data)
                    .await
            }
            Request::BootstrapEncryptedKey {
                nonce,
                ciphertext,
                associated_data,
            } => {
                self.handle_bootstrap_encrypted_key(nonce, ciphertext, associated_data)
                    .await
            }
            Request::QueryVaultState => self.handle_query_vault_state().await,
            Request::EvmWalletCreate => self.handle_evm_wallet_create().await,
            Request::EvmWalletList => self.handle_evm_wallet_list().await,
            Request::AuthChallengeRequest { .. }
            | Request::AuthChallengeSolution { .. }
            | Request::ClientConnectionResponse { .. } => {
                Err(TransportResponseError::UnexpectedRequestPayload)
            }
        }
    }
}

type Output = Result<Response, TransportResponseError>;

impl UserAgentSession {
    fn take_unseal_secret(
        &mut self,
    ) -> Result<(EphemeralSecret, PublicKey), TransportResponseError> {
        let UserAgentStates::WaitingForUnsealKey(unseal_context) = self.state.state() else {
            error!("Received encrypted key in invalid state");
            return Err(TransportResponseError::InvalidStateForUnsealEncryptedKey);
        };

        let ephemeral_secret = {
            let mut secret_lock = unseal_context.secret.lock().unwrap();
            let secret = secret_lock.take();
            match secret {
                Some(secret) => secret,
                None => {
                    drop(secret_lock);
                    error!("Ephemeral secret already taken");
                    return Err(TransportResponseError::StateTransitionFailed);
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
    ) -> Result<MemSafe<Vec<u8>>, ()> {
        let nonce = XNonce::from_slice(nonce);

        let shared_secret = ephemeral_secret.diffie_hellman(&client_public_key);
        let cipher = XChaCha20Poly1305::new(shared_secret.as_bytes().into());

        let mut key_buffer = MemSafe::new(ciphertext.to_vec()).unwrap();

        let decryption_result = {
            let mut write_handle = key_buffer.write().unwrap();
            let write_handle = write_handle.deref_mut();
            cipher.decrypt_in_place(nonce, associated_data, write_handle)
        };

        match decryption_result {
            Ok(_) => Ok(key_buffer),
            Err(err) => {
                error!(?err, "Failed to decrypt encrypted key material");
                Err(())
            }
        }
    }

    async fn handle_unseal_request(&mut self, client_pubkey: x25519_dalek::PublicKey) -> Output {
        let secret = EphemeralSecret::random();
        let public_key = PublicKey::from(&secret);

        self.transition(UserAgentEvents::UnsealRequest(UnsealContext {
            secret: Mutex::new(Some(secret)),
            client_public_key: client_pubkey,
        }))?;

        Ok(Response::UnsealStartResponse {
            server_pubkey: public_key,
        })
    }

    async fn handle_unseal_encrypted_key(
        &mut self,
        nonce: Vec<u8>,
        ciphertext: Vec<u8>,
        associated_data: Vec<u8>,
    ) -> Output {
        let (ephemeral_secret, client_public_key) = match self.take_unseal_secret() {
            Ok(values) => values,
            Err(TransportResponseError::StateTransitionFailed) => {
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                return Ok(Response::UnsealResult(Err(UnsealError::InvalidKey)));
            }
            Err(err) => return Err(err),
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
                return Ok(Response::UnsealResult(Err(UnsealError::InvalidKey)));
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
                info!("Successfully unsealed key with client-provided key");
                self.transition(UserAgentEvents::ReceivedValidKey)?;
                Ok(Response::UnsealResult(Ok(())))
            }
            Err(SendError::HandlerError(keyholder::Error::InvalidKey)) => {
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                Ok(Response::UnsealResult(Err(UnsealError::InvalidKey)))
            }
            Err(SendError::HandlerError(err)) => {
                error!(?err, "Keyholder failed to unseal key");
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                Ok(Response::UnsealResult(Err(UnsealError::InvalidKey)))
            }
            Err(err) => {
                error!(?err, "Failed to send unseal request to keyholder");
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                Err(TransportResponseError::KeyHolderActorUnreachable)
            }
        }
    }

    async fn handle_bootstrap_encrypted_key(
        &mut self,
        nonce: Vec<u8>,
        ciphertext: Vec<u8>,
        associated_data: Vec<u8>,
    ) -> Output {
        let (ephemeral_secret, client_public_key) = match self.take_unseal_secret() {
            Ok(values) => values,
            Err(TransportResponseError::StateTransitionFailed) => {
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                return Ok(Response::BootstrapResult(Err(BootstrapError::InvalidKey)));
            }
            Err(err) => return Err(err),
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
                return Ok(Response::BootstrapResult(Err(BootstrapError::InvalidKey)));
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
                info!("Successfully bootstrapped vault with client-provided key");
                self.transition(UserAgentEvents::ReceivedValidKey)?;
                Ok(Response::BootstrapResult(Ok(())))
            }
            Err(SendError::HandlerError(keyholder::Error::AlreadyBootstrapped)) => {
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                Ok(Response::BootstrapResult(Err(
                    BootstrapError::AlreadyBootstrapped,
                )))
            }
            Err(SendError::HandlerError(err)) => {
                error!(?err, "Keyholder failed to bootstrap vault");
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                Ok(Response::BootstrapResult(Err(BootstrapError::InvalidKey)))
            }
            Err(err) => {
                error!(?err, "Failed to send bootstrap request to keyholder");
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                Err(TransportResponseError::KeyHolderActorUnreachable)
            }
        }
    }
}

impl UserAgentSession {
    async fn handle_query_vault_state(&mut self) -> Output {
        use crate::actors::keyholder::{GetState, StateDiscriminants};

        let vault_state = match self.props.actors.key_holder.ask(GetState {}).await {
            Ok(StateDiscriminants::Unbootstrapped) => VaultState::Unbootstrapped,
            Ok(StateDiscriminants::Sealed) => VaultState::Sealed,
            Ok(StateDiscriminants::Unsealed) => VaultState::Unsealed,
            Err(err) => {
                error!(?err, actor = "useragent", "keyholder.query.failed");
                return Err(TransportResponseError::KeyHolderActorUnreachable);
            }
        };

        Ok(Response::VaultState(vault_state))
    }
}

impl UserAgentSession {
    async fn handle_evm_wallet_create(&mut self) -> Output {
        let result = match self.props.actors.evm.ask(Generate {}).await {
            Ok(_address) => return Ok(Response::EvmWalletCreate(Ok(()))),
            Err(SendError::HandlerError(err)) => Err(err),
            Err(err) => {
                error!(?err, "EVM actor unreachable during wallet create");
                return Err(TransportResponseError::KeyHolderActorUnreachable);
            }
        };
        Ok(Response::EvmWalletCreate(result))
    }

    async fn handle_evm_wallet_list(&mut self) -> Output {
        match self.props.actors.evm.ask(ListWallets {}).await {
            Ok(wallets) => Ok(Response::EvmWalletList(wallets)),
            Err(err) => {
                error!(?err, "EVM wallet list failed");
                Err(TransportResponseError::KeyHolderActorUnreachable)
            }
        }
    }
}
