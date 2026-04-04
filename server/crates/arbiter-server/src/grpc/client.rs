use arbiter_proto::{
    proto::client::{
        ClientRequest, ClientResponse, VaultState as ProtoVaultState,
        client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
    },
    transport::{Receiver, Sender, grpc::GrpcBi},
};
use kameo::{
    actor::{ActorRef, Spawn as _},
    error::SendError,
};
use tonic::Status;
use tracing::{info, warn};

use crate::{
    actors::{
        client::{
            self, ClientConnection,
            session::{ClientSession, Error, HandleQueryVaultState},
        },
        keyholder::KeyHolderState,
    },
    grpc::request_tracker::RequestTracker,
};

mod auth;
mod inbound;
mod outbound;

async fn dispatch_loop(
    mut bi: GrpcBi<ClientRequest, ClientResponse>,
    actor: ActorRef<ClientSession>,
    mut request_tracker: RequestTracker,
) {
    loop {
        let Some(message) = bi.recv().await else {
            return;
        };

        let conn = match message {
            Ok(conn) => conn,
            Err(err) => {
                warn!(error = ?err, "Failed to receive client request");
                return;
            }
        };

        let request_id = match request_tracker.request(conn.request_id) {
            Ok(id) => id,
            Err(err) => {
                let _ = bi.send(Err(err)).await;
                return;
            }
        };

        let Some(payload) = conn.payload else {
            let _ = bi
                .send(Err(Status::invalid_argument(
                    "Missing client request payload",
                )))
                .await;
            return;
        };

        match dispatch_inner(&actor, payload).await {
            Ok(response) => {
                if bi
                    .send(Ok(ClientResponse {
                        request_id: Some(request_id),
                        payload: Some(response),
                    }))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            Err(status) => {
                let _ = bi.send(Err(status)).await;
                return;
            }
        }
    }
}

async fn dispatch_inner(
    actor: &ActorRef<ClientSession>,
    payload: ClientRequestPayload,
) -> Result<ClientResponsePayload, Status> {
    match payload {
        ClientRequestPayload::QueryVaultState(_) => {
            let state = match actor.ask(HandleQueryVaultState {}).await {
                Ok(KeyHolderState::Unbootstrapped) => ProtoVaultState::Unbootstrapped,
                Ok(KeyHolderState::Sealed) => ProtoVaultState::Sealed,
                Ok(KeyHolderState::Unsealed) => ProtoVaultState::Unsealed,
                Err(SendError::HandlerError(Error::Internal)) => ProtoVaultState::Error,
                Err(err) => {
                    warn!(error = ?err, "Failed to query vault state");
                    ProtoVaultState::Error
                }
            };
            Ok(ClientResponsePayload::VaultState(state.into()))
        }
        payload => {
            warn!(?payload, "Unsupported post-auth client request");
            Err(Status::invalid_argument("Unsupported client request"))
        }
    }
}

pub async fn start(mut conn: ClientConnection, mut bi: GrpcBi<ClientRequest, ClientResponse>) {
    let mut request_tracker = RequestTracker::default();

    if let Err(e) = auth::start(&mut conn, &mut bi, &mut request_tracker).await {
        let mut transport = auth::AuthTransportAdapter::new(&mut bi, &mut request_tracker);
        let _ = transport.send(Err(e.clone())).await;
        warn!(error = ?e, "Client authentication failed");
        return;
    };

    let actor = client::session::ClientSession::spawn(client::session::ClientSession::new(conn));
    let actor_for_cleanup = actor.clone();

    info!("Client authenticated successfully");
    dispatch_loop(bi, actor, request_tracker).await;
    actor_for_cleanup.kill();
}
