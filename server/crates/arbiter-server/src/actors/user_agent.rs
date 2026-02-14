use arbiter_proto::proto::{
    UserAgentResponse,
    auth::{
        self, AuthChallenge, AuthChallengeRequest, AuthOk, ClientMessage,
        ServerMessage as AuthServerMessage, client_message::Payload as ClientAuthPayload,
        server_message::Payload as ServerAuthPayload,
    },
    user_agent_request::Payload as UserAgentRequestPayload,
    user_agent_response::Payload as UserAgentResponsePayload,
};
use diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl, dsl::update};
use diesel_async::{AsyncConnection, RunQueryDsl};
use ed25519_dalek::VerifyingKey;
use kameo::{
    Actor,
    actor::{ActorRef, Spawn},
    messages,
    prelude::Context,
};
use tokio::sync::mpsc::Sender;
use tonic::Status;
use tracing::{error, info};

use crate::{
    ServerContext,
    actors::bootstrap::{BootstrapActor, ConsumeToken},
    db::{self, schema},
    errors::GrpcStatusExt,
};

/// Context for state machine with validated key and sent challenge
/// Challenge is then transformed to bytes using shared function and verified
#[derive(Clone, Debug)]
pub struct ChallengeContext {
    challenge: AuthChallenge,
    key: VerifyingKey,
}

// Request context with deserialized public key for state machine.
// This intermediate struct is needed because the state machine branches depending on presence of bootstrap token,
// but we want to have the deserialized key in both branches.
#[derive(Clone, Debug)]
pub struct AuthRequestContext {
    pubkey: VerifyingKey,
    bootstrap_token: Option<String>,
}

smlang::statemachine!(
    name: UserAgent,
    derive_states: [Debug],
    custom_error: false,
    transitions: {
        *Init + AuthRequest(AuthRequestContext) / auth_request_context =  ReceivedAuthRequest(AuthRequestContext),
        ReceivedAuthRequest(AuthRequestContext) + ReceivedBootstrapToken = Idle,

        ReceivedAuthRequest(AuthRequestContext) + SentChallenge(ChallengeContext) / move_challenge = WaitingForChallengeSolution(ChallengeContext),

        WaitingForChallengeSolution(ChallengeContext) + ReceivedGoodSolution = Idle,
        WaitingForChallengeSolution(ChallengeContext) + ReceivedBadSolution = AuthError, // block further transitions, but connection should close anyway

        Idle + UnsealRequest / generate_temp_keypair = UnsealStarted(ed25519_dalek::SigningKey),
        UnsealStarted(ed25519_dalek::SigningKey) + SentTempKeypair / move_keypair = WaitingForUnsealKey(ed25519_dalek::SigningKey),
    }
);

pub struct DummyContext;
impl UserAgentStateMachineContext for DummyContext {
    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    fn move_challenge(
        &mut self,
        _state_data: &AuthRequestContext,
        event_data: ChallengeContext,
    ) -> Result<ChallengeContext, ()> {
        Ok(event_data)
    }

    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    fn auth_request_context(
        &mut self,
        event_data: AuthRequestContext,
    ) -> Result<AuthRequestContext, ()> {
        Ok(event_data)
    }

    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    fn move_keypair(
        &mut self,
        state_data: &ed25519_dalek::SigningKey,
    ) -> Result<ed25519_dalek::SigningKey, ()> {
        Ok(state_data.clone())
    }

    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    fn generate_temp_keypair(&mut self) -> Result<ed25519_dalek::SigningKey, ()> {
        Ok(ed25519_dalek::SigningKey::generate(&mut rand::rng()))
    }
}

#[derive(Actor)]
pub struct UserAgentActor {
    db: db::DatabasePool,
    bootstapper: ActorRef<BootstrapActor>,
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
        bootstapper: ActorRef<BootstrapActor>,
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

#[messages]
impl UserAgentActor {
    #[message(ctx)]
    pub async fn handle_auth_challenge_request(
        &mut self,
        req: AuthChallengeRequest,
        ctx: &mut Context<Self, Output>,
    ) -> Output {
        let pubkey = req.pubkey.as_array().ok_or(Status::invalid_argument(
            "Expected pubkey to have specific length",
        ))?;
        let pubkey = VerifyingKey::from_bytes(pubkey).map_err(|err| {
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

    #[message(ctx)]
    pub async fn handle_auth_challenge_solution(
        &mut self,
        solution: auth::AuthChallengeSolution,
        ctx: &mut Context<Self, Output>,
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

#[cfg(test)]
mod tests {
    use arbiter_proto::proto::{
        UserAgentResponse,
        auth::{self, AuthChallengeRequest, AuthOk},
        user_agent_response::Payload as UserAgentResponsePayload,
    };
    use chrono::format;
    use diesel::{ExpressionMethods as _, QueryDsl, insert_into};
    use diesel_async::RunQueryDsl;
    use ed25519_dalek::Signer as _;
    use kameo::actor::Spawn;

    use crate::{
        actors::{
            bootstrap::BootstrapActor,
            user_agent::{HandleAuthChallengeRequest, HandleAuthChallengeSolution},
        },
        db::{self, schema},
    };

    use super::UserAgentActor;

    #[tokio::test]
    #[test_log::test]
    pub async fn test_bootstrap_token_auth() {
        let db = db::create_test_pool().await;
        // explicitly not installing any user_agent pubkeys
        let bootstrapper = BootstrapActor::new(&db).await.unwrap(); // this will create bootstrap token
        let token = bootstrapper.get_token().unwrap();

        let bootstrapper_ref = BootstrapActor::spawn(bootstrapper);
        let user_agent = UserAgentActor::new_manual(
            db.clone(),
            bootstrapper_ref,
            tokio::sync::mpsc::channel(1).0, // dummy channel, we won't actually send responses in this test
        );
        let user_agent_ref = UserAgentActor::spawn(user_agent);

        // simulate client sending auth request with bootstrap token
        let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
        let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

        let result = user_agent_ref
            .ask(HandleAuthChallengeRequest {
                req: AuthChallengeRequest {
                    pubkey: pubkey_bytes,
                    bootstrap_token: Some(token),
                },
            })
            .await
            .expect("Shouldn't fail to send message");

        // auth succeeded
        assert_eq!(
            result,
            UserAgentResponse {
                payload: Some(UserAgentResponsePayload::AuthMessage(
                    arbiter_proto::proto::auth::ServerMessage {
                        payload: Some(arbiter_proto::proto::auth::server_message::Payload::AuthOk(
                            AuthOk {},
                        )),
                    },
                )),
            }
        );

        // key is succesfully recorded in database
        let mut conn = db.get().await.unwrap();
        let stored_pubkey: Vec<u8> = schema::useragent_client::table
            .select(schema::useragent_client::public_key)
            .first::<Vec<u8>>(&mut conn)
            .await
            .unwrap();
        assert_eq!(stored_pubkey, new_key.verifying_key().to_bytes().to_vec());
    }

    #[tokio::test]
    #[test_log::test]
    pub async fn test_bootstrap_invalid_token_auth() {
        let db = db::create_test_pool().await;
        // explicitly not installing any user_agent pubkeys
        let bootstrapper = BootstrapActor::new(&db).await.unwrap(); // this will create bootstrap token

        let bootstrapper_ref = BootstrapActor::spawn(bootstrapper);
        let user_agent = UserAgentActor::new_manual(
            db.clone(),
            bootstrapper_ref,
            tokio::sync::mpsc::channel(1).0, // dummy channel, we won't actually send responses in this test
        );
        let user_agent_ref = UserAgentActor::spawn(user_agent);

        // simulate client sending auth request with bootstrap token
        let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
        let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

        let result = user_agent_ref
            .ask(HandleAuthChallengeRequest {
                req: AuthChallengeRequest {
                    pubkey: pubkey_bytes,
                    bootstrap_token: Some("invalid_token".to_string()),
                },
            })
            .await;

        match result {
            Err(kameo::error::SendError::HandlerError(status)) => {
                assert_eq!(status.code(), tonic::Code::InvalidArgument);
                insta::assert_debug_snapshot!(status, @r#"
                Status {
                    code: InvalidArgument,
                    message: "Invalid bootstrap token",
                    source: None,
                }
                "#);
            }
            Err(other) => {
                panic!("Expected SendError::HandlerError, got {other:?}");
            }
            Ok(_) => {
                panic!("Expected error due to invalid bootstrap token, but got success");
            }
        }
    }

    #[tokio::test]
    #[test_log::test]
    pub async fn test_challenge_auth() {
        let db = db::create_test_pool().await;

        let bootstrapper_ref = BootstrapActor::spawn(BootstrapActor::new(&db).await.unwrap());
        let user_agent = UserAgentActor::new_manual(
            db.clone(),
            bootstrapper_ref,
            tokio::sync::mpsc::channel(1).0, // dummy channel, we won't actually send responses in this test
        );
        let user_agent_ref = UserAgentActor::spawn(user_agent);

        // simulate client sending auth request with bootstrap token
        let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
        let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

        // insert pubkey into database to trigger challenge-response auth flow
        {
            let mut conn = db.get().await.unwrap();
            insert_into(schema::useragent_client::table)
                .values(schema::useragent_client::public_key.eq(pubkey_bytes.clone()))
                .execute(&mut conn)
                .await
                .unwrap();
        }

        let result = user_agent_ref
            .ask(HandleAuthChallengeRequest {
                req: AuthChallengeRequest {
                    pubkey: pubkey_bytes,
                    bootstrap_token: None,
                },
            })
            .await
            .expect("Shouldn't fail to send message");

        // auth challenge succeeded
        let UserAgentResponse {
            payload:
                Some(UserAgentResponsePayload::AuthMessage(arbiter_proto::proto::auth::ServerMessage {
                    payload:
                        Some(arbiter_proto::proto::auth::server_message::Payload::AuthChallenge(
                            challenge,
                        )),
                })),
        } = result
        else {
            panic!("Expected auth challenge response, got {result:?}");
        };

        let formatted_challenge = arbiter_proto::format_challenge(&challenge);
        let signature = new_key.sign(&formatted_challenge);
        let serialized_signature = signature.to_bytes().to_vec();

        let result = user_agent_ref
            .ask(HandleAuthChallengeSolution {
                solution: auth::AuthChallengeSolution {
                    signature: serialized_signature,
                },
            })
            .await
            .expect("Shouldn't fail to send message");

        // auth succeeded
        assert_eq!(
            result,
            UserAgentResponse {
                payload: Some(UserAgentResponsePayload::AuthMessage(
                    arbiter_proto::proto::auth::ServerMessage {
                        payload: Some(arbiter_proto::proto::auth::server_message::Payload::AuthOk(
                            AuthOk {},
                        )),
                    },
                )),
            }
        );
    }
}

mod transport;
pub(crate) use transport::handle_user_agent;
