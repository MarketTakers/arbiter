use arbiter_proto::proto::client::{ClientRequest, ClientResponse};

use std::sync::atomic::{AtomicI32, Ordering};
use tokio::sync::mpsc;

pub const BUFFER_LENGTH: usize = 16;
static NEXT_REQUEST_ID: AtomicI32 = AtomicI32::new(1);

pub fn next_request_id() -> i32 {
    NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, thiserror::Error)]
pub enum ClientSignError {
    #[error("Transport channel closed")]
    ChannelClosed,

    #[error("Connection closed by server")]
    ConnectionClosed,
}

pub struct ClientTransport {
    pub(crate) sender: mpsc::Sender<ClientRequest>,
    pub(crate) receiver: tonic::Streaming<ClientResponse>,
}

impl ClientTransport {
    pub(crate) async fn send(&mut self, request: ClientRequest) -> Result<(), ClientSignError> {
        self.sender
            .send(request)
            .await
            .map_err(|_| ClientSignError::ChannelClosed)
    }

    pub(crate) async fn recv(&mut self) -> Result<ClientResponse, ClientSignError> {
        match self.receiver.message().await {
            Ok(Some(resp)) => Ok(resp),
            Ok(None) | Err(_) => Err(ClientSignError::ConnectionClosed),
        }
    }
}
