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
    utils::defer,
};

mod auth;

async fn dispatch_loop(
    mut bi: GrpcBi<ClientRequest, ClientResponse>,
    actor: ActorRef<ClientSession>,
    mut request_tracker: RequestTracker,
) {
    loop {
        let Some(conn) = bi.recv().await else {
            return;
        };

        if dispatch_conn_message(&mut bi, &actor, &mut request_tracker, conn)
            .await
            .is_err()
        {
            return;
        }
    }
}

async fn dispatch_conn_message(
    bi: &mut GrpcBi<ClientRequest, ClientResponse>,
    actor: &ActorRef<ClientSession>,
    request_tracker: &mut RequestTracker,
    conn: Result<ClientRequest, Status>,
) -> Result<(), ()> {
    let conn = match conn {
        Ok(conn) => conn,
        Err(err) => {
            warn!(error = ?err, "Failed to receive client request");
            return Err(());
        }
    };

    let request_id = match request_tracker.request(conn.request_id) {
        Ok(request_id) => request_id,
        Err(err) => {
            let _ = bi.send(Err(err)).await;
            return Err(());
        }
    };
    let Some(payload) = conn.payload else {
        let _ = bi
            .send(Err(Status::invalid_argument(
                "Missing client request payload",
            )))
            .await;
        return Err(());
    };

    let payload = match payload {
        ClientRequestPayload::QueryVaultState(_) => ClientResponsePayload::VaultState(
            match actor.ask(HandleQueryVaultState {}).await {
                Ok(KeyHolderState::Unbootstrapped) => ProtoVaultState::Unbootstrapped,
                Ok(KeyHolderState::Sealed) => ProtoVaultState::Sealed,
                Ok(KeyHolderState::Unsealed) => ProtoVaultState::Unsealed,
                Err(SendError::HandlerError(Error::Internal)) => ProtoVaultState::Error,
                Err(err) => {
                    warn!(error = ?err, "Failed to query vault state");
                    ProtoVaultState::Error
                }
            }
            .into(),
        ),
        payload => {
            warn!(?payload, "Unsupported post-auth client request");
            let _ = bi
                .send(Err(Status::invalid_argument("Unsupported client request")))
                .await;
            return Err(());
        }
    };

    bi.send(Ok(ClientResponse {
        request_id: Some(request_id),
        payload: Some(payload),
    }))
    .await
    .map_err(|_| ())
}

pub async fn start(conn: ClientConnection, mut bi: GrpcBi<ClientRequest, ClientResponse>) {
    let mut conn = conn;
    let mut request_tracker = RequestTracker::default();
    let mut response_id = None;

    match auth::start(&mut conn, &mut bi, &mut request_tracker, &mut response_id).await {
        Ok(_) => {
            let actor =
                client::session::ClientSession::spawn(client::session::ClientSession::new(conn));
            let actor_for_cleanup = actor.clone();
            let _ = defer(move || {
                actor_for_cleanup.kill();
            });

            info!("Client authenticated successfully");
            dispatch_loop(bi, actor, request_tracker).await;
        }
        Err(e) => {
            let mut transport = auth::AuthTransportAdapter::new(
                &mut bi,
                &mut request_tracker,
                &mut response_id,
            );
            let _ = transport.send(Err(e.clone())).await;
            warn!(error = ?e, "Authentication failed");
            return;
        }
    }
}
