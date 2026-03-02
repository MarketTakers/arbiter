use std::{ops::DerefMut, sync::Mutex};

use arbiter_proto::proto::user_agent::{
    UnsealEncryptedKey, UnsealResult, UnsealStart, UnsealStartResponse, UserAgentRequest,
    UserAgentResponse, user_agent_request::Payload as UserAgentRequestPayload,
    user_agent_response::Payload as UserAgentResponsePayload,
};
use chacha20poly1305::{AeadInPlace, XChaCha20Poly1305, XNonce, aead::KeyInit};
use ed25519_dalek::VerifyingKey;
use kameo::{Actor, error::SendError};
use memsafe::MemSafe;
use tokio::select;
use tracing::{error, info};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::actors::{
    keyholder::{self, TryUnseal},
    user_agent::{ConnectionProps, UserAgentError},
};

mod state;
use state::{DummyContext, UnsealContext, UserAgentEvents, UserAgentStateMachine, UserAgentStates};

pub struct UserAgentSession {
    props: ConnectionProps,
    key: VerifyingKey,
    state: UserAgentStateMachine<DummyContext>,
}

impl UserAgentSession {
    pub(crate) fn new(props: ConnectionProps, key: VerifyingKey) -> Self {
        Self {
            props,
            key,
            state: UserAgentStateMachine::new(DummyContext),
        }
    }

    fn transition(&mut self, event: UserAgentEvents) -> Result<(), UserAgentError> {
        self.state.process_event(event).map_err(|e| {
            error!(?e, "State transition failed");
            UserAgentError::StateTransitionFailed
        })?;
        Ok(())
    }

    pub async fn process_transport_inbound(&mut self, req: UserAgentRequest) -> Output {
        let msg = req.payload.ok_or_else(|| {
            error!(actor = "useragent", "Received message with no payload");
            UserAgentError::MissingRequestPayload
        })?;

        match msg {
            UserAgentRequestPayload::UnsealStart(unseal_start) => {
                self.handle_unseal_request(unseal_start).await
            }
            UserAgentRequestPayload::UnsealEncryptedKey(unseal_encrypted_key) => {
                self.handle_unseal_encrypted_key(unseal_encrypted_key).await
            }
            _ => Err(UserAgentError::UnexpectedRequestPayload),
        }
    }
}

type Output = Result<UserAgentResponse, UserAgentError>;

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
            .map_err(|_| UserAgentError::InvalidClientPubkeyLength)?;

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
            return Err(UserAgentError::InvalidStateForUnsealEncryptedKey);
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
                        Err(UserAgentError::KeyHolderActorUnreachable)
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

impl Actor for UserAgentSession {
    type Args = Self;

    type Error = ();

    async fn on_start(
        args: Self::Args,
        _: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
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
        let props = ConnectionProps::new(db, actors, transport);
        let key = VerifyingKey::from_bytes(&[0u8; 32]).unwrap();
        Self {
            props,
            key,
            state: UserAgentStateMachine::new(DummyContext),
        }
    }
}
