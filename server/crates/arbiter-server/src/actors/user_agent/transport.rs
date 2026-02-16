use super::UserAgentActor;
use arbiter_proto::proto::{
    UserAgentRequest, UserAgentResponse,
    auth::{ClientMessage as ClientAuthMessage, client_message::Payload as ClientAuthPayload},
    user_agent_request::Payload as UserAgentRequestPayload,
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
    actors::user_agent::{
        HandleAuthChallengeRequest, HandleAuthChallengeSolution, HandleUnsealEncryptedKey,
        HandleUnsealRequest,
    },
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

    match msg {
        UserAgentRequestPayload::AuthMessage(ClientAuthMessage {
            payload: Some(ClientAuthPayload::AuthChallengeRequest(req)),
        }) => actor
            .ask(HandleAuthChallengeRequest { req })
            .await
            .map_err(into_status),
        UserAgentRequestPayload::AuthMessage(ClientAuthMessage {
            payload: Some(ClientAuthPayload::AuthChallengeSolution(solution)),
        }) => actor
            .ask(HandleAuthChallengeSolution { solution })
            .await
            .map_err(into_status),
        UserAgentRequestPayload::UnsealStart(unseal_start) => actor
            .ask(HandleUnsealRequest { req: unseal_start })
            .await
            .map_err(into_status),
        UserAgentRequestPayload::UnsealEncryptedKey(unseal_encrypted_key) => actor
            .ask(HandleUnsealEncryptedKey {
                req: unseal_encrypted_key,
            })
            .await
            .map_err(into_status),
        _ => Err(Status::invalid_argument("Expected message with payload")),
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
