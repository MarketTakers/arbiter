use std::sync::Arc;

use arbiter_proto::{
    proto::{
        UserAgentRequest, UserAgentResponse,
        auth::{
            self, AuthChallengeRequest, ClientMessage, ServerMessage as AuthServerMessage,
            client_message::Payload as ClientAuthPayload,
            server_message::Payload as ServerAuthPayload,
        },
        user_agent_request::Payload as UserAgentRequestPayload,
        user_agent_response::Payload as UserAgentResponsePayload,
    },
    transport::Bi,
};
use diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use ed25519_dalek::{SigningKey, VerifyingKey};
use futures::StreamExt;
use kameo::{
    Actor,
    actor::{ActorRef, Spawn},
    error::SendError,
    message::StreamMessage,
    messages,
    prelude::Context,
};
use secrecy::{ExposeSecret, SecretBox};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tonic::{Status, transport::Server};
use tracing::error;

use crate::{ServerContext, context::bootstrap::ConsumeToken, db::schema};

#[derive(Debug)]
pub struct ChallengeContext {
    challenge: auth::AuthChallenge,
    key: SigningKey,
    bootstrap_token: Option<String>,
}

smlang::statemachine!(
    name: UserAgent,
    derive_states: [Debug],
    transitions: {
        *Init + ReceivedRequest(ed25519_dalek::VerifyingKey) [async check_key_existence] / provide_challenge = WaitingForChallengeSolution(ChallengeContext),
        Init + ReceivedBootstrapToken(String) = Authenticated,

        WaitingForChallengeSolution(ChallengeContext) + ReceivedGoodSolution = Authenticated,
        WaitingForChallengeSolution(ChallengeContext) + ReceivedBadSolution = Error,
    }
);

impl UserAgentStateMachineContext for ServerContext {
    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    fn provide_challenge(
        &mut self,
        event_data: ed25519_dalek::VerifyingKey,
    ) -> Result<ChallengeContext, ()> {
        todo!()
    }

    #[allow(missing_docs)]
    #[allow(clippy::result_unit_err)]
    async fn check_key_existence(
        &self,
        event_data: &ed25519_dalek::VerifyingKey,
    ) -> Result<bool, ()> {
        todo!()
    }
}

#[derive(Actor)]
pub struct UserAgentActor {
    context: ServerContext,
    state: UserAgentStateMachine<ServerContext>,
    rx: Sender<Result<UserAgentResponse, Status>>,
}

impl UserAgentActor {
    pub(crate) fn new(
        context: ServerContext,
        rx: Sender<Result<UserAgentResponse, Status>>,
    ) -> Self {
        Self {
            context: context.clone(),
            state: UserAgentStateMachine::new(context),
            rx,
        }
    }

    async fn auth_with_bootstrap_token(
        &mut self,
        pubkey: ed25519_dalek::VerifyingKey,
        token: String,
    ) -> Result<UserAgentResponse, Status> {
        let token_ok: bool = self
            .context
            .bootstrapper
            .ask(ConsumeToken { token })
            .await
            .map_err(|e| {
                error!(?pubkey, "Failed to consume bootstrap token: {e}");
                Status::internal("Bootstrap token consumption failed")
            })?;

        if token_ok {
            let mut conn = self.context.db.get().await.map_err(|e| {
                error!(?pubkey, "Failed to get DB connection: {e}");
                Status::internal("Database connection error")
            })?;

            diesel::insert_into(schema::useragent_client::table)
                .values((
                    schema::useragent_client::public_key.eq(pubkey.as_bytes().to_vec()),
                    schema::useragent_client::nonce.eq(1),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| {
                    error!(?pubkey, "Failed to insert new user agent client: {e}");
                    Status::internal("Database error")
                })?;

            return Ok(UserAgentResponse {
                payload: Some(UserAgentResponsePayload::AuthMessage(AuthServerMessage {
                    payload: Some(ServerAuthPayload::Auth),
                })),
            });
        }

        todo!()
    }
}

type Output = Result<UserAgentResponse, Status>;

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

        if let Some(token) = req.bootstrap_token {
            return self.auth_with_bootstrap_token(pubkey, token).await;
        }

        let mut db_conn = self.context.db.get().await.map_err(|err| {
            error!(?pubkey, "Failed to get DB connection: {err}");
            Status::internal("Database connection error")
        })?;

        let nonce = db_conn
            .transaction(|mut conn| {
                Box::pin(async move {
                    let current_nonce = schema::useragent_client::table
                        .filter(schema::useragent_client::public_key.eq(pubkey.as_bytes().to_vec()))
                        .select(schema::useragent_client::nonce)
                        .first::<i32>(&mut db_conn)
                        .await?;

                    Ok(())
                })
            })
            .await;

        // let nonce = match last_used_nonce

        todo!()
    }

    #[message(ctx)]
    pub async fn handle_auth_challenge_solution(
        &mut self,
        solution: auth::AuthChallengeSolution,
        ctx: &mut Context<Self, Output>,
    ) -> Output {
        todo!()
    }
}

pub(crate) async fn handle_user_agent(
    context: ServerContext,
    mut req_stream: tonic::Streaming<UserAgentRequest>,
    tx: mpsc::Sender<Result<UserAgentResponse, Status>>,
) {
    let actor = UserAgentActor::spawn(UserAgentActor::new(context, tx.clone()));

    while let Some(Ok(req)) = req_stream.next().await
        && actor.is_alive()
    {
        match process_message(&actor, req).await {
            Ok(resp) => {
                if tx.send(Ok(resp)).await.is_err() {
                    error!(actor = "useragent", "Failed to send response to client");
                    break;
                }
            }
            Err(status) => {
                let _ = tx.send(Err(status)).await;
                break;
            }
        }
    }

    actor.kill();
}

async fn process_message(
    actor: &ActorRef<UserAgentActor>,
    req: UserAgentRequest,
) -> Result<UserAgentResponse, Status> {
    let msg = req.payload.ok_or_else(|| {
        error!(actor = "useragent", "Received message with no payload");
        Status::invalid_argument("Expected message with payload")
    })?;

    let UserAgentRequestPayload::AuthMessage(ClientMessage {
        payload: Some(client_message),
    }) = msg
    else {
        error!(
            actor = "useragent",
            "Received unexpected message type during authentication"
        );
        return Err(Status::invalid_argument(
            "Expected AuthMessage with ClientMessage payload",
        ));
    };

    let result = match client_message {
        ClientAuthPayload::AuthChallengeRequest(req) => actor
            .ask(HandleAuthChallengeRequest { req })
            .await
            .map_err(into_status),
        ClientAuthPayload::AuthChallengeSolution(solution) => actor
            .ask(HandleAuthChallengeSolution { solution })
            .await
            .map_err(into_status),
    };

    result
}

fn into_status<M>(e: SendError<M, Status>) -> Status {
    match e {
        SendError::HandlerError(status) => status,
        _ => {
            error!(actor = "useragent", "Failed to send message to actor");
            Status::internal("session failure")
        }
    }
}
