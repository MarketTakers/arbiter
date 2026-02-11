use arbiter_proto::{
    proto::{
        UserAgentRequest, UserAgentResponse,
        auth::{
            self, AuthChallengeRequest, ClientMessage, client_message::Payload as ClientAuthPayload
        },
        user_agent_request::Payload as UserAgentRequestPayload,
    },
    transport::Bi,
};
use futures::StreamExt;
use tracing::error;

use crate::ServerContext;

smlang::statemachine!(
    name: UserAgentAuth,
    derive_states: [Debug],
    derive_events: [Clone, Debug],
    transitions: {
        *Init + ReceivedRequest(ed25519_dalek::VerifyingKey) / provide_challenge = WaitingForChallengeSolution(auth::AuthChallenge),
        WaitingForChallengeSolution(auth::AuthChallenge) + ReceivedGoodSolution = Authenticated,
        WaitingForChallengeSolution(auth::AuthChallenge) + ReceivedBadSolution = Error,
    }
);



impl UserAgentAuthStateMachineContext for ServerContext {
    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    fn provide_challenge< >(&mut self,_event_data:ed25519_dalek::VerifyingKey) -> Result<auth::AuthChallenge,()>  {
        todo!()
    }
}

pub(crate) async fn handle_user_agent(
    context: ServerContext,
    mut bistream: impl Bi<UserAgentRequest, UserAgentResponse> + Unpin,
) {
    let auth_sm = UserAgentAuthStateMachine::new(context);

    while let Some(Ok(msg)) = bistream.next().await
        && auth_sm.state() != &UserAgentAuthStates::Authenticated
    {
        let Some(msg) = msg.payload else {
            error!(handler = "useragent", "Received message with no payload");
            return;
        };

        let UserAgentRequestPayload::AuthMessage(ClientMessage {
            payload: Some(client_message),
        }) = msg
        else {
            error!(
                handler = "useragent",
                "Received unexpected message type during authentication"
            );
            return;
        };

        match client_message {
            ClientAuthPayload::AuthChallengeRequest(auth_challenge_request) => {
                let AuthChallengeRequest { pubkey  } = auth_challenge_request;
            },
            ClientAuthPayload::AuthChallengeSolution(_auth_challenge_solution) => todo!(),
        }
    }
}
