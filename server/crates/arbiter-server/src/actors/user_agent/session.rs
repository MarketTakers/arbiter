use std::{ops::DerefMut, sync::Mutex};

use arbiter_proto::proto::{
    evm as evm_proto,
    user_agent::{
        ClientConnectionCancel, ClientConnectionRequest, UnsealEncryptedKey, UnsealResult,
        UnsealStart, UnsealStartResponse, UserAgentRequest, UserAgentResponse,
        user_agent_request::Payload as UserAgentRequestPayload,
        user_agent_response::Payload as UserAgentResponsePayload,
    },
};
use chacha20poly1305::{AeadInPlace, XChaCha20Poly1305, XNonce, aead::KeyInit};
use ed25519_dalek::VerifyingKey;
use kameo::{Actor, error::SendError, messages, prelude::Context};
use memsafe::MemSafe;
use tokio::{select, sync::watch};
use tracing::{error, info};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::actors::{
    evm::{Generate, ListWallets},
    keyholder::{self, TryUnseal},
    router::RegisterUserAgent,
    user_agent::{TransportResponseError, UserAgentConnection},
};

mod state;
use state::{DummyContext, UnsealContext, UserAgentEvents, UserAgentStateMachine, UserAgentStates};

// Error for consumption by other actors
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("User agent session ended due to connection loss")]
    ConnectionLost,

    #[error("User agent session ended due to unexpected message")]
    UnexpectedMessage,
}

pub struct UserAgentSession {
    props: UserAgentConnection,
    state: UserAgentStateMachine<DummyContext>,
}

impl UserAgentSession {
    pub(crate) fn new(props: UserAgentConnection) -> Self {
        Self {
            props,
            state: UserAgentStateMachine::new(DummyContext),
        }
    }

    fn transition(&mut self, event: UserAgentEvents) -> Result<(), TransportResponseError> {
        self.state.process_event(event).map_err(|e| {
            error!(?e, "State transition failed");
            TransportResponseError::StateTransitionFailed
        })?;
        Ok(())
    }

    async fn send_msg<Reply: kameo::Reply>(
        &mut self,
        msg: UserAgentResponsePayload,
        _ctx: &mut Context<Self, Reply>,
    ) -> Result<(), Error> {
        self.props
            .transport
            .send(Ok(response(msg)))
            .await
            .map_err(|_| {
                error!(
                    actor = "useragent",
                    reason = "channel closed",
                    "send.failed"
                );
                Error::ConnectionLost
            })
    }

    async fn expect_msg<Extractor, Msg, Reply>(
        &mut self,
        extractor: Extractor,
        ctx: &mut Context<Self, Reply>,
    ) -> Result<Msg, Error>
    where
        Extractor: FnOnce(UserAgentRequestPayload) -> Option<Msg>,
        Reply: kameo::Reply,
    {
        let msg = self.props.transport.recv().await.ok_or_else(|| {
            error!(
                actor = "useragent",
                reason = "channel closed",
                "recv.failed"
            );
            ctx.stop();
            Error::ConnectionLost
        })?;

        msg.payload.and_then(extractor).ok_or_else(|| {
            error!(
                actor = "useragent",
                reason = "unexpected message",
                "recv.failed"
            );
            ctx.stop();
            Error::UnexpectedMessage
        })
    }
}

#[messages]
impl UserAgentSession {
    // TODO: Think about refactoring it to state-machine based flow, as we already have one
    #[message(ctx)]
    pub async fn request_new_client_approval(
        &mut self,
        client_pubkey: VerifyingKey,
        mut cancel_flag: watch::Receiver<()>,
        ctx: &mut Context<Self, Result<bool, Error>>,
    ) -> Result<bool, Error> {
        self.send_msg(
            UserAgentResponsePayload::ClientConnectionRequest(ClientConnectionRequest {
                pubkey: client_pubkey.as_bytes().to_vec(),
            }),
            ctx,
        )
        .await?;

        let extractor = |msg| {
            if let UserAgentRequestPayload::ClientConnectionResponse(client_connection_response) =
                msg
            {
                Some(client_connection_response)
            } else {
                None
            }
        };

        tokio::select! {
            _ = cancel_flag.changed() => {
                info!(actor = "useragent", "client connection approval cancelled");
                self.send_msg(
                    UserAgentResponsePayload::ClientConnectionCancel(ClientConnectionCancel {}),
                    ctx,
                ).await?;
                Ok(false)
            }
            result = self.expect_msg(extractor, ctx) => {
                let result = result?;
                info!(actor = "useragent", "received client connection approval result: approved={}", result.approved);
                Ok(result.approved)
            }
        }
    }
}

impl UserAgentSession {
    pub async fn process_transport_inbound(&mut self, req: UserAgentRequest) -> Output {
        let msg = req.payload.ok_or_else(|| {
            error!(actor = "useragent", "Received message with no payload");
            TransportResponseError::MissingRequestPayload
        })?;

        match msg {
            UserAgentRequestPayload::UnsealStart(unseal_start) => {
                self.handle_unseal_request(unseal_start).await
            }
            UserAgentRequestPayload::UnsealEncryptedKey(unseal_encrypted_key) => {
                self.handle_unseal_encrypted_key(unseal_encrypted_key).await
            }
            UserAgentRequestPayload::EvmWalletCreate(_) => self.handle_evm_wallet_create().await,
            UserAgentRequestPayload::EvmWalletList(_) => self.handle_evm_wallet_list().await,
            _ => Err(TransportResponseError::UnexpectedRequestPayload),
        }
    }
}

type Output = Result<UserAgentResponse, TransportResponseError>;

fn response(payload: UserAgentResponsePayload) -> UserAgentResponse {
    UserAgentResponse {
        payload: Some(payload),
    }
}

impl UserAgentSession {
    async fn handle_unseal_request(&mut self, req: UnsealStart) -> Output {
        let secret = EphemeralSecret::random();
        let public_key = PublicKey::from(&secret);

        let client_pubkey_bytes: [u8; 32] = req
            .client_pubkey
            .try_into()
            .map_err(|_| TransportResponseError::InvalidClientPubkeyLength)?;

        let client_public_key = PublicKey::from(client_pubkey_bytes);

        self.transition(UserAgentEvents::UnsealRequest(UnsealContext {
            secret: Mutex::new(Some(secret)),
            client_public_key,
        }))?;

        Ok(response(UserAgentResponsePayload::UnsealStartResponse(
            UnsealStartResponse {
                server_pubkey: public_key.as_bytes().to_vec(),
            },
        )))
    }

    async fn handle_unseal_encrypted_key(&mut self, req: UnsealEncryptedKey) -> Output {
        let UserAgentStates::WaitingForUnsealKey(unseal_context) = self.state.state() else {
            error!("Received unseal encrypted key in invalid state");
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
                    self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                    return Ok(response(UserAgentResponsePayload::UnsealResult(
                        UnsealResult::InvalidKey.into(),
                    )));
                }
            }
        };

        let nonce = XNonce::from_slice(&req.nonce);

        let shared_secret = ephemeral_secret.diffie_hellman(&unseal_context.client_public_key);
        let cipher = XChaCha20Poly1305::new(shared_secret.as_bytes().into());

        let mut seal_key_buffer = MemSafe::new(req.ciphertext.clone()).unwrap();

        let decryption_result = {
            let mut write_handle = seal_key_buffer.write().unwrap();
            let write_handle = write_handle.deref_mut();
            cipher.decrypt_in_place(nonce, &req.associated_data, write_handle)
        };

        match decryption_result {
            Ok(_) => {
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
                        Ok(response(UserAgentResponsePayload::UnsealResult(
                            UnsealResult::Success.into(),
                        )))
                    }
                    Err(SendError::HandlerError(keyholder::Error::InvalidKey)) => {
                        self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                        Ok(response(UserAgentResponsePayload::UnsealResult(
                            UnsealResult::InvalidKey.into(),
                        )))
                    }
                    Err(SendError::HandlerError(err)) => {
                        error!(?err, "Keyholder failed to unseal key");
                        self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                        Ok(response(UserAgentResponsePayload::UnsealResult(
                            UnsealResult::InvalidKey.into(),
                        )))
                    }
                    Err(err) => {
                        error!(?err, "Failed to send unseal request to keyholder");
                        self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                        Err(TransportResponseError::KeyHolderActorUnreachable)
                    }
                }
            }
            Err(err) => {
                error!(?err, "Failed to decrypt unseal key");
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                Ok(response(UserAgentResponsePayload::UnsealResult(
                    UnsealResult::InvalidKey.into(),
                )))
            }
        }
    }
}

impl UserAgentSession {
    async fn handle_evm_wallet_create(&mut self) -> Output {
        use evm_proto::wallet_create_response::Result as CreateResult;

        let result = match self.props.actors.evm.ask(Generate {}).await {
            Ok(address) => CreateResult::Wallet(evm_proto::WalletEntry {
                address: address.as_slice().to_vec(),
            }),
            Err(err) => CreateResult::Error(map_evm_error("wallet create", err).into()),
        };

        Ok(response(UserAgentResponsePayload::EvmWalletCreate(
            evm_proto::WalletCreateResponse {
                result: Some(result),
            },
        )))
    }

    async fn handle_evm_wallet_list(&mut self) -> Output {
        use evm_proto::wallet_list_response::Result as ListResult;

        let result = match self.props.actors.evm.ask(ListWallets {}).await {
            Ok(wallets) => ListResult::Wallets(evm_proto::WalletList {
                wallets: wallets
                    .into_iter()
                    .map(|addr| evm_proto::WalletEntry {
                        address: addr.as_slice().to_vec(),
                    })
                    .collect(),
            }),
            Err(err) => ListResult::Error(map_evm_error("wallet list", err).into()),
        };

        Ok(response(UserAgentResponsePayload::EvmWalletList(
            evm_proto::WalletListResponse {
                result: Some(result),
            },
        )))
    }
}

fn map_evm_error<M>(op: &str, err: SendError<M, crate::actors::evm::Error>) -> evm_proto::EvmError {
    use crate::actors::{evm::Error as EvmError, keyholder::Error as KhError};
    match err {
        SendError::HandlerError(EvmError::Keyholder(KhError::NotBootstrapped)) => {
            evm_proto::EvmError::VaultSealed
        }
        SendError::HandlerError(err) => {
            error!(?err, "EVM {op} failed");
            evm_proto::EvmError::Internal
        }
        _ => {
            error!("EVM actor unreachable during {op}");
            evm_proto::EvmError::Internal
        }
    }
}

impl Actor for UserAgentSession {
    type Args = Self;

    type Error = TransportResponseError;

    async fn on_start(
        args: Self::Args,
        this: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        args.props
            .actors
            .router
            .ask(RegisterUserAgent {
                actor: this.clone(),
            })
            .await
            .map_err(|err| {
                error!(?err, "Failed to register user agent connection with router");
                TransportResponseError::ConnectionRegistrationFailed
            })?;
        Ok(args)
    }

    async fn next(
        &mut self,
        _actor_ref: kameo::prelude::WeakActorRef<Self>,
        mailbox_rx: &mut kameo::prelude::MailboxReceiver<Self>,
    ) -> Option<kameo::mailbox::Signal<Self>> {
        loop {
            select! {
                signal = mailbox_rx.recv() => {
                    return signal;
                }
                msg = self.props.transport.recv() => {
                    match msg {
                        Some(request) => {
                            match self.process_transport_inbound(request).await {
                                Ok(response) => {
                                    if self.props.transport.send(Ok(response)).await.is_err() {
                                        error!(actor = "useragent", reason = "channel closed", "send.failed");
                                        return Some(kameo::mailbox::Signal::Stop);
                                    }
                                }
                                Err(err) => {
                                    let _ = self.props.transport.send(Err(err)).await;
                                    return Some(kameo::mailbox::Signal::Stop);
                                }
                            }
                        }
                        None => {
                            info!(actor = "useragent", "transport.closed");
                            return Some(kameo::mailbox::Signal::Stop);
                        }
                    }
                }
            }
        }
    }
}

impl UserAgentSession {
    pub fn new_test(db: crate::db::DatabasePool, actors: crate::actors::GlobalActors) -> Self {
        use arbiter_proto::transport::DummyTransport;
        let transport: super::Transport = Box::new(DummyTransport::new());
        let props = UserAgentConnection::new(db, actors, transport);
        Self {
            props,
            state: UserAgentStateMachine::new(DummyContext),
        }
    }
}
