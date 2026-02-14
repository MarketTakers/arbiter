use super::UserAgentActor;
use arbiter_proto::proto::{
    UserAgentRequest, UserAgentResponse,
    auth::{
        self, AuthChallenge, AuthChallengeRequest, AuthOk, ClientMessage,
        ServerMessage as AuthServerMessage, client_message::Payload as ClientAuthPayload,
        server_message::Payload as ServerAuthPayload,
    },
    user_agent_request::Payload as UserAgentRequestPayload,
    user_agent_response::Payload as UserAgentResponsePayload,
};
use futures::StreamExt;
use kameo::{
    actor::{ActorRef, Spawn as _},
    error::SendError,
};
use tokio::sync::mpsc;
use tonic::Status;
use tracing::error;

use crate::{
    actors::user_agent::{HandleAuthChallengeRequest, HandleAuthChallengeSolution},
    context::ServerContext,
};

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

    match client_message {
        ClientAuthPayload::AuthChallengeRequest(req) => actor
            .ask(HandleAuthChallengeRequest { req })
            .await
            .map_err(into_status),
        ClientAuthPayload::AuthChallengeSolution(solution) => actor
            .ask(HandleAuthChallengeSolution { solution })
            .await
            .map_err(into_status),
    }
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
