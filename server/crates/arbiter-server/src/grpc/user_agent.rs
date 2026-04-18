use tokio::sync::mpsc;

use arbiter_proto::{
    proto::user_agent::{
        UserAgentRequest, UserAgentResponse,
        user_agent_request::Payload as UserAgentRequestPayload,
        user_agent_response::Payload as UserAgentResponsePayload,
    },
    transport::{Error as TransportError, Receiver, Sender, grpc::GrpcBi},
};
use async_trait::async_trait;
use kameo::actor::ActorRef;
use tonic::Status;
use tracing::{error, info, warn};

use crate::{
    grpc::request_tracker::RequestTracker,
    peers::user_agent::{OutOfBand, UserAgentConnection, UserAgentSession},
};

mod auth;
mod evm;
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
    mut bi: GrpcBi<UserAgentRequest, UserAgentResponse>,
    actor: ActorRef<UserAgentSession>,
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

                if bi.send(Ok(UserAgentResponse { id: None, payload: Some(payload) })).await.is_err() {
                    return;
                }
            }

            message = bi.recv() => {
                let Some(message) = message else { return; };

                let conn = match message {
                    Ok(conn) => conn,
                    Err(err) => {
                        warn!(error = ?err, "Failed to receive user agent request");
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
                    let _ = bi.send(Err(Status::invalid_argument("Missing user-agent request payload"))).await;
                    return;
                };

                match dispatch_inner(&actor, payload).await {
                    Ok(Some(response)) => {
                        if bi.send(Ok(UserAgentResponse {
                            id: Some(request_id),
                            payload: Some(response),
                        })).await.is_err() {
                            return;
                        }
                    }
                    Ok(None) => {}
                    Err(status) => {
                        error!(?status, "Failed to process user agent request");
                        let _ = bi.send(Err(status)).await;
                        return;
                    }
                }
            }
        }
    }
}

async fn dispatch_inner(
    actor: &ActorRef<UserAgentSession>,
    payload: UserAgentRequestPayload,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    match payload {
        UserAgentRequestPayload::Vault(req) => vault::dispatch(actor, req).await,
        UserAgentRequestPayload::Evm(req) => evm::dispatch(actor, req).await,
        UserAgentRequestPayload::SdkClient(req) => sdk_client::dispatch(actor, req).await,
        UserAgentRequestPayload::Auth(..) => {
            warn!("Unsupported post-auth user agent auth request");
            Err(Status::invalid_argument("Unsupported user-agent request"))
        }
    }
}

pub async fn start(
    mut conn: UserAgentConnection,
    mut bi: GrpcBi<UserAgentRequest, UserAgentResponse>,
) {
    let mut request_tracker = RequestTracker::default();

    let (oob_sender, oob_receiver) = mpsc::channel(16);
    let oob_adapter = OutOfBandAdapter(oob_sender);

    let actor = {
        let transport = auth::AuthTransportAdapter::new(&mut bi, &mut request_tracker);
        match crate::peers::user_agent::start(&mut conn, transport, Box::new(oob_adapter)).await {
            Ok(actor) => actor,
            Err(e) => {
                warn!(error = ?e, "User agent connection failed");
                return;
            }
        }
    };

    info!("User agent session established");

    dispatch_loop(bi, actor.clone(), oob_receiver, request_tracker).await;
    actor.kill();
}

