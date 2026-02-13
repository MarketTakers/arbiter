use std::sync::Arc;

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
use ed25519_dalek::VerifyingKey;
use futures::StreamExt;
use kameo::{Actor, message::StreamMessage, messages, prelude::Context};
use secrecy::{ExposeSecret, SecretBox};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
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
        todo!()
    }
}

type Output = Result<UserAgentResponse, Status>;

#[messages]
impl UserAgentActor {
    #[message(ctx)]
    async fn handle_auth_challenge_request(
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
            return self
                .auth_with_bootstrap_token(pubkey, token)
                .await
                .map_err(|_| Status::internal("Failed to authenticate with bootstrap token"));
        }

        todo!()
    }

    #[message(ctx)]
    async fn handle_auth_challenge_solution(
        &mut self,
        _solution: auth::AuthChallengeSolution,
        ctx: &mut Context<Self, Output>,
    ) -> Output {
        todo!()
    }
}
