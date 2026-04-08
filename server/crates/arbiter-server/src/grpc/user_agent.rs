use tokio::sync::{mpsc, oneshot};

use arbiter_proto::{
    proto::user_agent::{
        UserAgentRequest, UserAgentResponse,
        user_agent_request::Payload as UserAgentRequestPayload,
        user_agent_response::Payload as UserAgentResponsePayload,
    },
    transport::{Error as TransportError, Receiver, Sender, grpc::GrpcBi},
};
use async_trait::async_trait;
use kameo::actor::{ActorRef, Spawn as _};
use tonic::Status;
use tracing::{error, info, warn};

use crate::{
    crypto::integrity,
    grpc::request_tracker::RequestTracker,
    peers::user_agent::{
        Credentials, OutOfBand, UserAgentConnection, UserAgentSession,
        vault_gate::VaultGate,
    },
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

    let auth_creds = match auth::start(&mut conn, &mut bi, &mut request_tracker).await {
        Ok(creds) => creds,
        Err(e) => {
            warn!(error = ?e, "Authentication failed");
            return;
        }
    };

    info!(pubkey = ?auth_creds.creds.pubkey, "User authenticated successfully");

    let creds = if integrity::is_signing_available(&conn.actors.vault)
        .await
        .unwrap_or(false)
    {
        // Vault is unsealed; integrity was verified during auth — promote directly.
        auth_creds.creds
    } else {
        // Vault is sealed/unbootstrapped; run the VaultGate phase.
        let (promotion_tx, promotion_rx) = oneshot::channel();
        let gate = VaultGate::spawn(VaultGate::new(
            auth_creds,
            conn.actors.clone(),
            conn.db.clone(),
            promotion_tx,
        ));

        let result = vault_gate_loop(&mut bi, &gate, &mut request_tracker, promotion_rx).await;
        gate.kill();

        match result {
            Some(creds) => creds,
            None => return,
        }
    };

    let (oob_sender, oob_receiver) = mpsc::channel(16);
    let oob_adapter = OutOfBandAdapter(oob_sender);

    let actor = UserAgentSession::spawn(UserAgentSession::new(conn, creds, Box::new(oob_adapter)));
    let actor_for_cleanup = actor.clone();

    dispatch_loop(bi, actor, oob_receiver, request_tracker).await;
    actor_for_cleanup.kill();
}

async fn vault_gate_loop(
    bi: &mut GrpcBi<UserAgentRequest, UserAgentResponse>,
    gate: &ActorRef<VaultGate>,
    request_tracker: &mut RequestTracker,
    mut promotion_rx: oneshot::Receiver<Result<Credentials, crate::peers::user_agent::vault_gate::Error>>,
) -> Option<Credentials> {
    loop {
        tokio::select! {
            result = &mut promotion_rx => {
                return match result {
                    Ok(Ok(creds)) => Some(creds),
                    Ok(Err(e)) => {
                        warn!(error = ?e, "VaultGate promotion failed");
                        None
                    }
                    Err(_) => {
                        warn!("VaultGate promotion channel closed unexpectedly");
                        None
                    }
                };
            }

            message = bi.recv() => {
                let Some(message) = message else { return None; };

                let conn = match message {
                    Ok(conn) => conn,
                    Err(err) => {
                        warn!(error = ?err, "Failed to receive request during vault gate phase");
                        return None;
                    }
                };

                let request_id = match request_tracker.request(conn.id) {
                    Ok(id) => id,
                    Err(err) => {
                        let _ = bi.send(Err(err)).await;
                        return None;
                    }
                };

                let Some(payload) = conn.payload else {
                    let _ = bi.send(Err(Status::invalid_argument("Missing request payload"))).await;
                    return None;
                };

                let response = match payload {
                    UserAgentRequestPayload::Vault(req) => vault_gate::dispatch(gate, req).await,
                    _ => Err(Status::permission_denied("Only vault operations are permitted before unsealing")),
                };

                match response {
                    Ok(Some(payload)) => {
                        if bi.send(Ok(UserAgentResponse { id: Some(request_id), payload: Some(payload) })).await.is_err() {
                            return None;
                        }
                    }
                    Ok(None) => {}
                    Err(status) => {
                        let _ = bi.send(Err(status)).await;
                        return None;
                    }
                }
            }
        }
    }
}
