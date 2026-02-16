use std::{
    ops::DerefMut,
    sync::Mutex,
};

use arbiter_proto::proto::{
    UserAgentResponse,
    auth::{
        self, AuthChallengeRequest, AuthOk, ServerMessage as AuthServerMessage,
        server_message::Payload as ServerAuthPayload,
    },
    unseal::{UnsealEncryptedKey, UnsealResult, UnsealStart, UnsealStartResponse},
    user_agent_response::Payload as UserAgentResponsePayload,
};
use chacha20poly1305::{
    AeadInPlace, XChaCha20Poly1305, XNonce,
    aead::KeyInit,
};
use diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl, dsl::update};
use diesel_async::{AsyncConnection, RunQueryDsl};
use ed25519_dalek::VerifyingKey;
use kameo::{Actor, actor::ActorRef, messages};
use memsafe::MemSafe;
use tokio::sync::mpsc::Sender;
use tonic::Status;
use tracing::{error, info};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::{
    ServerContext,
    actors::{
        bootstrap::{Bootstrapper, ConsumeToken},
        user_agent::state::{
            AuthRequestContext, ChallengeContext, DummyContext, UnsealContext, UserAgentEvents,
            UserAgentStateMachine, UserAgentStates,
        },
    },
    db::{self, schema},
    errors::GrpcStatusExt,
};

mod state;
#[cfg(test)]
mod tests;

mod transport;
pub(crate) use transport::handle_user_agent;

#[derive(Actor)]
pub struct UserAgentActor {
    db: db::DatabasePool,
    bootstapper: ActorRef<Bootstrapper>,
    state: UserAgentStateMachine<DummyContext>,
    // will be used in future
    _tx: Sender<Result<UserAgentResponse, Status>>,
}

impl UserAgentActor {
    pub(crate) fn new(
        context: ServerContext,
        tx: Sender<Result<UserAgentResponse, Status>>,
    ) -> Self {
        Self {
            db: context.db.clone(),
            bootstapper: context.bootstrapper.clone(),
            state: UserAgentStateMachine::new(DummyContext),
            _tx: tx,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_manual(
        db: db::DatabasePool,
        bootstapper: ActorRef<Bootstrapper>,
        tx: Sender<Result<UserAgentResponse, Status>>,
    ) -> Self {
        Self {
            db,
            bootstapper,
            state: UserAgentStateMachine::new(DummyContext),
            _tx: tx,
        }
    }

    fn transition(&mut self, event: UserAgentEvents) -> Result<(), Status> {
        self.state.process_event(event).map_err(|e| {
            error!(?e, "State transition failed");
            Status::internal("State machine error")
        })?;
        Ok(())
    }

    async fn auth_with_bootstrap_token(
        &mut self,
        pubkey: ed25519_dalek::VerifyingKey,
        token: String,
    ) -> Result<UserAgentResponse, Status> {
        let token_ok: bool = self
            .bootstapper
            .ask(ConsumeToken { token })
            .await
            .map_err(|e| {
                error!(?pubkey, "Failed to consume bootstrap token: {e}");
                Status::internal("Bootstrap token consumption failed")
            })?;

        if !token_ok {
            error!(?pubkey, "Invalid bootstrap token provided");
            return Err(Status::invalid_argument("Invalid bootstrap token"));
        }

        {
            let mut conn = self.db.get().await.to_status()?;

            diesel::insert_into(schema::useragent_client::table)
                .values((
                    schema::useragent_client::public_key.eq(pubkey.as_bytes().to_vec()),
                    schema::useragent_client::nonce.eq(1),
                ))
                .execute(&mut conn)
                .await
                .to_status()?;
        }

        self.transition(UserAgentEvents::ReceivedBootstrapToken)?;

        Ok(auth_response(ServerAuthPayload::AuthOk(AuthOk {})))
    }

    async fn auth_with_challenge(&mut self, pubkey: VerifyingKey, pubkey_bytes: Vec<u8>) -> Output {
        let nonce: Option<i32> = {
            let mut db_conn = self.db.get().await.to_status()?;
            db_conn
                .transaction(|conn| {
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
                .to_status()?
        };

        let Some(nonce) = nonce else {
            error!(?pubkey, "Public key not found in database");
            return Err(Status::unauthenticated("Public key not registered"));
        };

        let challenge = auth::AuthChallenge {
            pubkey: pubkey_bytes,
            nonce: nonce,
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
    ) -> Result<(bool, &ChallengeContext), Status> {
        let UserAgentStates::WaitingForChallengeSolution(challenge_context) = self.state.state()
        else {
            error!("Received challenge solution in invalid state");
            return Err(Status::invalid_argument(
                "Invalid state for challenge solution",
            ));
        };
        let formatted_challenge = arbiter_proto::format_challenge(&challenge_context.challenge);

        let signature = solution.signature.as_slice().try_into().map_err(|_| {
            error!(?solution, "Invalid signature length");
            Status::invalid_argument("Invalid signature length")
        })?;

        let valid = challenge_context
            .key
            .verify_strict(&formatted_challenge, &signature)
            .is_ok();

        Ok((valid, challenge_context))
    }
}

type Output = Result<UserAgentResponse, Status>;

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

#[messages]
impl UserAgentActor {
    #[message]
    pub async fn handle_unseal_request(&mut self, req: UnsealStart) -> Output {
        let secret = EphemeralSecret::random();
        let public_key = PublicKey::from(&secret);

        let client_pubkey_bytes: [u8; 32] = req
            .client_pubkey
            .try_into()
            .map_err(|_| Status::invalid_argument("client_pubkey must be 32 bytes"))?;

        let client_public_key = PublicKey::from(client_pubkey_bytes);

        self.transition(UserAgentEvents::UnsealRequest(UnsealContext {
            server_public_key: public_key,
            secret: Mutex::new(Some(secret)),
            client_public_key,
        }))?;

        Ok(unseal_response(
            UserAgentResponsePayload::UnsealStartResponse(UnsealStartResponse {
                server_pubkey: public_key.as_bytes().to_vec(),
            }),
        ))
    }

    #[message]
    pub async fn handle_unseal_encrypted_key(&mut self, req: UnsealEncryptedKey) -> Output {
        let UserAgentStates::WaitingForUnsealKey(unseal_context) = self.state.state() else {
            error!("Received unseal encrypted key in invalid state");
            return Err(Status::failed_precondition(
                "Invalid state for unseal encrypted key",
            ));
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

        let mut root_key_buffer = MemSafe::new(req.ciphertext.clone()).unwrap();
        let mut write_handle = root_key_buffer.write().unwrap();
        let write_handle = write_handle.deref_mut();

        let decryption_result = cipher
            .decrypt_in_place(nonce, &req.associated_data, write_handle);

        match decryption_result {
            Ok(_) => todo!("Send key to the keyguarding"),
            Err(err) => {
                error!(?err, "Failed to decrypt unseal key");
                self.transition(UserAgentEvents::ReceivedInvalidKey)?;
                return Ok(unseal_response(UserAgentResponsePayload::UnsealResult(
                    UnsealResult::InvalidKey.into(),
                )));
            },
        }
    }

    #[message]
    pub async fn handle_auth_challenge_request(&mut self, req: AuthChallengeRequest) -> Output {
        let pubkey = req.pubkey.as_array().ok_or(Status::invalid_argument(
            "Expected pubkey to have specific length",
        ))?;
        let pubkey = VerifyingKey::from_bytes(pubkey).map_err(|_err| {
            error!(?pubkey, "Failed to convert to VerifyingKey");
            Status::invalid_argument("Failed to convert pubkey to VerifyingKey")
        })?;

        self.transition(UserAgentEvents::AuthRequest(AuthRequestContext {
            pubkey,
            bootstrap_token: req.bootstrap_token.clone(),
        }))?;

        match req.bootstrap_token {
            Some(token) => self.auth_with_bootstrap_token(pubkey, token).await,
            None => self.auth_with_challenge(pubkey, req.pubkey).await,
        }
    }

    #[message]
    pub async fn handle_auth_challenge_solution(
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
            Err(Status::unauthenticated("Invalid challenge solution"))
        }
    }
}
