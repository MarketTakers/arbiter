use arbiter_proto::{
    proto::{
        UserAgentRequest, UserAgentResponse,
        auth::{
            self, AuthChallengeRequest, ClientMessage, client_message::Payload as ClientAuthPayload,
        },
        user_agent_request::Payload as UserAgentRequestPayload,
    },
    transport::Bi,
};
use futures::StreamExt;
use kameo::{Actor, message::StreamMessage, messages, prelude::Context};
use secrecy::{ExposeSecret, SecretBox};
use tokio::sync::mpsc;
use tonic::{Status, transport::Server};
use tracing::error;

use crate::ServerContext;

#[derive(Debug)]
pub struct ChallengeContext {
    challenge: auth::AuthChallenge,
    key: ed25519_dalek::SigningKey,
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
    tx: mpsc::Sender<Result<UserAgentResponse, tonic::Status>>,
}

impl UserAgentActor {
    pub(crate) fn new(
        context: ServerContext,
        tx: mpsc::Sender<Result<UserAgentResponse, tonic::Status>>,
    ) -> Self {
        Self {
            context: context.clone(),
            state: UserAgentStateMachine::new(context),
            tx,
        }
    }

    async fn handle_grpc(
        &mut self,
        msg: UserAgentRequest,
        ctx: &mut Context<Self, ()>,
    ) -> Result<UserAgentResponse, tonic::Status> {
        let Some(msg) = msg.payload else {
            error!(actor = "useragent", "Received message with no payload");
            ctx.stop();
            return Err(tonic::Status::invalid_argument(
                "Message payload is required",
            ));
        };

        let UserAgentRequestPayload::AuthMessage(ClientMessage {
            payload: Some(client_message),
        }) = msg
        else {
            error!(
                actor = "useragent",
                "Received unexpected message type during authentication"
            );
            ctx.stop();
            return Err(tonic::Status::invalid_argument(
                "Unexpected message type during authentication",
            ));
        };

        match client_message {
            ClientAuthPayload::AuthChallengeRequest(AuthChallengeRequest {
                payload: Some(payload),
            }) => match payload {
                auth::auth_challenge_request::Payload::Pubkey(items) => todo!(),
                auth::auth_challenge_request::Payload::BootstrapToken(_) => todo!(),
            },
            ClientAuthPayload::AuthChallengeSolution(_auth_challenge_solution) => todo!(),
            _ => {
                error!(
                    actor = "useragent",
                    "Received unexpected message type during authentication"
                );
                ctx.stop();
                return Err(tonic::Status::invalid_argument(
                    "Unexpected message type during authentication",
                ));
            }
        }
    }
}

#[messages]
impl UserAgentActor {
    #[message(ctx)]
    pub async fn grpc(&mut self, msg: UserAgentRequest, ctx: &mut Context<Self, ()>) {
        let result = self.handle_grpc(msg, ctx).await;
        self.tx.send(result).await.unwrap_or_else(|e| {
            error!(handler = "useragent", "Failed to send response: {}", e);
            ctx.stop();
        });
    }
}
