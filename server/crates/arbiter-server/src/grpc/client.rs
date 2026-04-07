use arbiter_proto::{
    proto::client::{
        ClientRequest, ClientResponse, client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
    },
    transport::{Receiver, Sender, grpc::GrpcBi},
};
use kameo::actor::{ActorRef, Spawn as _};
use tonic::Status;
use tracing::{info, warn};

use crate::{
    peers::client::{ClientConnection, session::ClientSession},
    grpc::request_tracker::RequestTracker,
};

mod auth;
mod evm;
mod inbound;
mod outbound;
mod vault;

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
        ClientRequestPayload::Vault(req) => vault::dispatch(actor, req).await,
        ClientRequestPayload::Evm(req) => evm::dispatch(actor, req).await,
        ClientRequestPayload::Auth(..) => {
            warn!("Unsupported post-auth client auth request");
            Err(Status::invalid_argument("Unsupported client request"))
        }
    }
}

pub async fn start(mut conn: ClientConnection, mut bi: GrpcBi<ClientRequest, ClientResponse>) {
    let mut request_tracker = RequestTracker::default();

    let client_id = match auth::start(&mut conn, &mut bi, &mut request_tracker).await {
        Ok(id) => id,
        Err(err) => {
            let _ = bi
                .send(Err(Status::unauthenticated(format!(
                    "Authentication failed: {}",
                    err
                ))))
                .await;
            warn!(error = ?err, "Client authentication failed");
            return;
        }
    };

    let actor = ClientSession::spawn(ClientSession::new(conn, client_id));
    let actor_for_cleanup = actor.clone();

    info!("Client authenticated successfully");
    dispatch_loop(bi, actor, request_tracker).await;
    actor_for_cleanup.kill();
}
