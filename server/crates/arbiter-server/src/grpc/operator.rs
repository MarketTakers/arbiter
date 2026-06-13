use crate::{
    grpc::request_tracker::RequestTracker,
    peers::operator::{OutOfBand, OperatorConnection, OperatorSession},
};
use arbiter_proto::{
    proto::operator::{
        OperatorRequest, OperatorResponse,
        operator_request::Payload as OperatorRequestPayload,
        operator_response::Payload as OperatorResponsePayload,
    },
    transport::{Error as TransportError, Receiver, Sender, grpc::GrpcBi},
};

use async_trait::async_trait;
use kameo::actor::ActorRef;
use tokio::sync::mpsc;
use tonic::Status;
use tracing::{error, info, warn};

mod auth;
mod evm;
mod governance;
mod inbound;
mod outbound;
mod sdk_client;
mod vault;
mod vault_gate;

pub struct OutOfBandAdapter(mpsc::Sender<OutOfBand>);

#[async_trait]
impl Sender<OutOfBand> for OutOfBandAdapter {
    async fn send(&mut self, item: OutOfBand) -> Result<(), TransportError> {
        self.0.send(item).await.map_err(|e| {
            warn!(error = ?e, "Failed to send out-of-band message");
            TransportError::ChannelClosed
        })
    }
}

async fn dispatch_loop(
    mut bi: GrpcBi<OperatorRequest, OperatorResponse>,
    actor: ActorRef<OperatorSession>,
    mut receiver: mpsc::Receiver<OutOfBand>,
    mut request_tracker: RequestTracker,
) {
    loop {
        tokio::select! {
            oob = receiver.recv() => {
                let Some(oob) = oob else {
                    warn!("Out-of-band message channel closed");
                    return;
                };

                let payload = sdk_client::out_of_band_payload(oob);

                if bi.send(Ok(OperatorResponse { id: None, payload: Some(payload) })).await.is_err() {
                    return;
                }
            }

            message = bi.recv() => {
                let Some(message) = message else { return; };

                let conn = match message {
                    Ok(conn) => conn,
                    Err(err) => {
                        warn!(error = ?err, "Failed to receive operator request");
                        return;
                    }
                };

                let request_id = match request_tracker.request(conn.id) {
                    Ok(id) => id,
                    Err(err) => {
                        let _ = bi.send(Err(err)).await;
                        return;
                    }
                };

                let Some(payload) = conn.payload else {
                    let _ = bi.send(Err(Status::invalid_argument("Missing operator request payload"))).await;
                    return;
                };

                match dispatch_inner(&actor, payload).await {
                    Ok(Some(response)) => {
                        if bi.send(Ok(OperatorResponse {
                            id: Some(request_id),
                            payload: Some(response),
                        })).await.is_err() {
                            return;
                        }
                    }
                    Ok(None) => {}
                    Err(status) => {
                        error!(?status, "Failed to process operator request");
                        let _ = bi.send(Err(status)).await;
                        return;
                    }
                }
            }
        }
    }
}

async fn dispatch_inner(
    actor: &ActorRef<OperatorSession>,
    payload: OperatorRequestPayload,
) -> Result<Option<OperatorResponsePayload>, Status> {
    match payload {
        OperatorRequestPayload::Vault(req) => vault::dispatch(actor, req).await,
        OperatorRequestPayload::Evm(req) => evm::dispatch(actor, req).await,
        OperatorRequestPayload::SdkClient(req) => sdk_client::dispatch(actor, req).await,
        OperatorRequestPayload::Auth(..) => {
            warn!("Unsupported post-auth operator auth request");
            Err(Status::invalid_argument("Unsupported operator request"))
        }
        OperatorRequestPayload::Governance(req) => governance::dispatch(actor, req).await,
    }
}

pub async fn start(
    mut conn: OperatorConnection,
    mut bi: GrpcBi<OperatorRequest, OperatorResponse>,
) {
    let mut request_tracker = RequestTracker::default();

    let (oob_sender, oob_receiver) = mpsc::channel(16);
    let oob_adapter = OutOfBandAdapter(oob_sender);

    let actor = {
        let transport = auth::AuthTransportAdapter::new(&mut bi, &mut request_tracker);
        match crate::peers::operator::start(&mut conn, transport, Box::new(oob_adapter)).await {
            Ok(actor) => actor,
            Err(e) => {
                warn!(error = ?e, "Operator connection failed");
                return;
            }
        }
    };

    info!("Operator session established");

    dispatch_loop(bi, actor.clone(), oob_receiver, request_tracker).await;
    actor.kill();
}
