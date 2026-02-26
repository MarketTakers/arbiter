use std::{ops::DerefMut, sync::Mutex};

use arbiter_proto::{
    proto::{
        UnsealEncryptedKey, UnsealResult, UnsealStart, UnsealStartResponse, UserAgentRequest,
        UserAgentResponse,
        auth::{
            self, AuthChallengeRequest, AuthOk, ClientMessage as ClientAuthMessage,
            ServerMessage as AuthServerMessage, client_message::Payload as ClientAuthPayload,
            server_message::Payload as ServerAuthPayload,
        },
        user_agent_request::Payload as UserAgentRequestPayload,
        user_agent_response::Payload as UserAgentResponsePayload,
    },
    transport::{Bi, DummyTransport},
};
use chacha20poly1305::{AeadInPlace, XChaCha20Poly1305, XNonce, aead::KeyInit};
use diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl, dsl::update};
use diesel_async::RunQueryDsl;
use ed25519_dalek::VerifyingKey;
use kameo::{Actor, error::SendError};
use memsafe::MemSafe;
use tokio::select;
use tracing::{error, info};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::{
    ServerContext,
    actors::{
        GlobalActors,
        bootstrap::ConsumeToken,
        keyholder::{self, TryUnseal},
        user_agent::state::{
            ChallengeContext, DummyContext, UnsealContext, UserAgentEvents, UserAgentStateMachine,
            UserAgentStates,
        },
    },
    db::{self, schema},
};

mod state;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UserAgentError {
    #[error("Expected message with payload")]
    MissingRequestPayload,
    #[error("Expected message with payload")]
    UnexpectedRequestPayload,
    #[error("Invalid state for challenge solution")]
    InvalidStateForChallengeSolution,
    #[error("Invalid state for unseal encrypted key")]
    InvalidStateForUnsealEncryptedKey,
    #[error("client_pubkey must be 32 bytes")]
    InvalidClientPubkeyLength,
    #[error("Expected pubkey to have specific length")]
    InvalidAuthPubkeyLength,
    #[error("Failed to convert pubkey to VerifyingKey")]
    InvalidAuthPubkeyEncoding,
    #[error("Invalid signature length")]
    InvalidSignatureLength,
    #[error("Invalid bootstrap token")]
    InvalidBootstrapToken,
    #[error("Public key not registered")]
    PublicKeyNotRegistered,
    #[error("Invalid challenge solution")]
    InvalidChallengeSolution,
    #[error("State machine error")]
    StateTransitionFailed,
    #[error("Bootstrap token consumption failed")]
    BootstrapperActorUnreachable,
    #[error("Vault is not available")]
    KeyHolderActorUnreachable,
    #[error("Database pool error")]
    DatabasePoolUnavailable,
    #[error("Database error")]
    DatabaseOperationFailed,
}

pub struct UserAgentActor<Transport>
where
    Transport: Bi<UserAgentRequest, Result<UserAgentResponse, UserAgentError>>,
{
    db: db::DatabasePool,
    actors: GlobalActors,
    state: UserAgentStateMachine<DummyContext>,
    transport: Transport,
}

impl<Transport> UserAgentActor<Transport>
where
    Transport: Bi<UserAgentRequest, Result<UserAgentResponse, UserAgentError>>,
{
    pub(crate) fn new(context: ServerContext, transport: Transport) -> Self {
        Self {
            db: context.db.clone(),
            actors: context.actors.clone(),
            state: UserAgentStateMachine::new(DummyContext),
            transport,
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
            UserAgentRequestPayload::AuthMessage(ClientAuthMessage {
                payload: Some(ClientAuthPayload::AuthChallengeRequest(req)),
            }) => self.handle_auth_challenge_request(req).await,
            UserAgentRequestPayload::AuthMessage(ClientAuthMessage {
                payload: Some(ClientAuthPayload::AuthChallengeSolution(solution)),
            }) => self.handle_auth_challenge_solution(solution).await,
            UserAgentRequestPayload::UnsealStart(unseal_start) => {
                self.handle_unseal_request(unseal_start).await
            }
            UserAgentRequestPayload::UnsealEncryptedKey(unseal_encrypted_key) => {
                self.handle_unseal_encrypted_key(unseal_encrypted_key).await
            }
            _ => Err(UserAgentError::UnexpectedRequestPayload),
        }
    }

    async fn auth_with_bootstrap_token(
        &mut self,
        pubkey: ed25519_dalek::VerifyingKey,
        token: String,
    ) -> Result<UserAgentResponse, UserAgentError> {
        let token_ok: bool = self
            .actors
            .bootstrapper
            .ask(ConsumeToken { token })
            .await
            .map_err(|e| {
                error!(?pubkey, "Failed to consume bootstrap token: {e}");
                UserAgentError::BootstrapperActorUnreachable
            })?;

        if !token_ok {
            error!(?pubkey, "Invalid bootstrap token provided");
            return Err(UserAgentError::InvalidBootstrapToken);
        }

        {
            let mut conn = self.db.get().await.map_err(|e| {
                error!(error = ?e, "Database pool error");
                UserAgentError::DatabasePoolUnavailable
            })?;

            diesel::insert_into(schema::useragent_client::table)
                .values((
                    schema::useragent_client::public_key.eq(pubkey.as_bytes().to_vec()),
                    schema::useragent_client::nonce.eq(1),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| {
                    error!(error = ?e, "Database error");
                    UserAgentError::DatabaseOperationFailed
                })?;
        }

        self.transition(UserAgentEvents::ReceivedBootstrapToken)?;

        Ok(auth_response(ServerAuthPayload::AuthOk(AuthOk {})))
    }

    async fn auth_with_challenge(&mut self, pubkey: VerifyingKey, pubkey_bytes: Vec<u8>) -> Output {
        let nonce: Option<i32> = {
            let mut db_conn = self.db.get().await.map_err(|e| {
                error!(error = ?e, "Database pool error");
                UserAgentError::DatabasePoolUnavailable
            })?;
            db_conn
                .exclusive_transaction(|conn| {
                    Box::pin(async move {
                        let current_nonce = schema::useragent_client::table
                            .filter(
                                schema::useragent_client::public_key.eq(pubkey.as_bytes().to_vec()),
                            )
                            .select(schema::useragent_client::nonce)
                            .first::<i32>(conn)
                            .await?;

                        update(schema::useragent_client::table)
                            .filter(
                                schema::useragent_client::public_key.eq(pubkey.as_bytes().to_vec()),
                            )
                            .set(schema::useragent_client::nonce.eq(current_nonce + 1))
                            .execute(conn)
                            .await?;

                        Result::<_, diesel::result::Error>::Ok(current_nonce)
                    })
                })
                .await
                .optional()
                .map_err(|e| {
                    error!(error = ?e, "Database error");
                    UserAgentError::DatabaseOperationFailed
                })?
        };

        let Some(nonce) = nonce else {
            error!(?pubkey, "Public key not found in database");
            return Err(UserAgentError::PublicKeyNotRegistered);
        };

        let challenge = auth::AuthChallenge {
            pubkey: pubkey_bytes,
            nonce,
        };

        self.transition(UserAgentEvents::SentChallenge(ChallengeContext {
            challenge: challenge.clone(),
            key: pubkey,
        }))?;

        info!(
            ?pubkey,
            ?challenge,
            "Sent authentication challenge to client"
        );

        Ok(auth_response(ServerAuthPayload::AuthChallenge(challenge)))
    }

    fn verify_challenge_solution(
        &self,
        solution: &auth::AuthChallengeSolution,
    ) -> Result<(bool, &ChallengeContext), UserAgentError> {
        let UserAgentStates::WaitingForChallengeSolution(challenge_context) = self.state.state()
        else {
            error!("Received challenge solution in invalid state");
            return Err(UserAgentError::InvalidStateForChallengeSolution);
        };
        let formatted_challenge = arbiter_proto::format_challenge(&challenge_context.challenge);

        let signature = solution.signature.as_slice().try_into().map_err(|_| {
            error!(?solution, "Invalid signature length");
            UserAgentError::InvalidSignatureLength
        })?;

        let valid = challenge_context
            .key
            .verify_strict(&formatted_challenge, &signature)
            .is_ok();

        Ok((valid, challenge_context))
    }
}

type Output = Result<UserAgentResponse, UserAgentError>;

fn auth_response(payload: ServerAuthPayload) -> UserAgentResponse {
    UserAgentResponse {
        payload: Some(UserAgentResponsePayload::AuthMessage(AuthServerMessage {
            payload: Some(payload),
        })),
    }
}

fn unseal_response(payload: UserAgentResponsePayload) -> UserAgentResponse {
    UserAgentResponse {
        payload: Some(payload),
    }
}

impl<Transport> UserAgentActor<Transport>
where
    Transport: Bi<UserAgentRequest, Result<UserAgentResponse, UserAgentError>>,
{
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

        Ok(unseal_response(
            UserAgentResponsePayload::UnsealStartResponse(UnsealStartResponse {
                server_pubkey: public_key.as_bytes().to_vec(),
            }),
        ))
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
                    return Ok(unseal_response(UserAgentResponsePayload::UnsealResult(
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
                        Ok(unseal_response(UserAgentResponsePayload::UnsealResult(
                            UnsealResult::Success.into(),
                        )))
                    }
                    Err(SendError::HandlerError(keyholder::Error::InvalidKey)) => {
                        self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                        Ok(unseal_response(UserAgentResponsePayload::UnsealResult(
                            UnsealResult::InvalidKey.into(),
                        )))
                    }
                    Err(SendError::HandlerError(err)) => {
                        error!(?err, "Keyholder failed to unseal key");
                        self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                        Ok(unseal_response(UserAgentResponsePayload::UnsealResult(
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
                Ok(unseal_response(UserAgentResponsePayload::UnsealResult(
                    UnsealResult::InvalidKey.into(),
                )))
            }
        }
    }

    async fn handle_auth_challenge_request(&mut self, req: AuthChallengeRequest) -> Output {
        let pubkey = req
            .pubkey
            .as_array()
            .ok_or(UserAgentError::InvalidAuthPubkeyLength)?;
        let pubkey = VerifyingKey::from_bytes(pubkey).map_err(|_err| {
            error!(?pubkey, "Failed to convert to VerifyingKey");
            UserAgentError::InvalidAuthPubkeyEncoding
        })?;

        self.transition(UserAgentEvents::AuthRequest)?;

        match req.bootstrap_token {
            Some(token) => self.auth_with_bootstrap_token(pubkey, token).await,
            None => self.auth_with_challenge(pubkey, req.pubkey).await,
        }
    }

    async fn handle_auth_challenge_solution(
        &mut self,
        solution: auth::AuthChallengeSolution,
    ) -> Output {
        let (valid, challenge_context) = self.verify_challenge_solution(&solution)?;

        if valid {
            info!(
                ?challenge_context,
                "Client provided valid solution to authentication challenge"
            );
            self.transition(UserAgentEvents::ReceivedGoodSolution)?;
            Ok(auth_response(ServerAuthPayload::AuthOk(AuthOk {})))
        } else {
            error!("Client provided invalid solution to authentication challenge");
            self.transition(UserAgentEvents::ReceivedBadSolution)?;
            Err(UserAgentError::InvalidChallengeSolution)
        }
    }
}


impl<Transport> Actor for UserAgentActor<Transport>
where
    Transport: Bi<UserAgentRequest, Result<UserAgentResponse, UserAgentError>>,
{
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
                msg = self.transport.recv() => {
                    match msg {
                        Some(request) => {
                            match self.process_transport_inbound(request).await {
                                Ok(response) => {
                                    if self.transport.send(Ok(response)).await.is_err() {
                                        error!(actor = "useragent", reason = "channel closed", "send.failed");
                                        return Some(kameo::mailbox::Signal::Stop);
                                    }
                                }
                                Err(err) => {
                                    let _ = self.transport.send(Err(err)).await;
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


impl UserAgentActor<DummyTransport<UserAgentRequest, Result<UserAgentResponse, UserAgentError>>> {
    pub fn new_manual(db: db::DatabasePool, actors: GlobalActors) -> Self {
        Self {
            db,
            actors,
            state: UserAgentStateMachine::new(DummyContext),
            transport: DummyTransport::new(),
        }
    }
}
