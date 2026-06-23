use super::auth::AuthTransportAdapter;
use crate::{
    grpc::TryConvert,
    peers::operator::vault_gate::{self as vault_gate},
};
use arbiter_proto::transport::{Bi, Error as TransportError, Receiver, Sender};

use async_trait::async_trait;
use tonic::Status;
use tracing::warn;

mod inbound;
mod outbound;

#[async_trait]
impl Receiver<vault_gate::Inbound> for AuthTransportAdapter<'_> {
    async fn recv(&mut self) -> Option<vault_gate::Inbound> {
        let request = match self.bi_mut().recv().await? {
            Ok(request) => request,
            Err(error) => {
                warn!(
                    ?error,
                    "Failed to receive operator request during vault gate"
                );
                return None;
            }
        };

        if let Err(err) = self.tracker_mut().request(request.id) {
            let _ = self.bi_mut().send(Err(err)).await;
            return None;
        }

        let Some(payload) = request.payload else {
            let _ = self
                .bi_mut()
                .send(Err(Status::invalid_argument("Missing request payload")))
                .await;
            return None;
        };

        match payload.try_convert() {
            Ok(inbound) => Some(inbound),
            Err(status) => {
                let _ = self.bi_mut().send(Err(status)).await;
                None
            }
        }
    }
}

#[async_trait]
impl Sender<Result<vault_gate::Outbound, vault_gate::Error>> for AuthTransportAdapter<'_> {
    async fn send(
        &mut self,
        item: Result<vault_gate::Outbound, vault_gate::Error>,
    ) -> Result<(), TransportError> {
        let outbound = match item {
            Ok(outbound) => outbound,
            Err(err) => {
                warn!(?err, "vault gate produced transport-level error");
                return self
                    .bi_mut()
                    .send(Err(Status::internal(err.to_string())))
                    .await;
            }
        };

        match outbound.try_convert() {
            Ok(payload) => self.send_response_payload(payload).await,
            Err(status) => self.bi_mut().send(Err(status)).await,
        }
    }
}

impl Bi<vault_gate::Inbound, Result<vault_gate::Outbound, vault_gate::Error>>
    for AuthTransportAdapter<'_>
{
}
