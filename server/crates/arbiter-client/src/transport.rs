use arbiter_proto::proto::client::{ClientRequest, ClientResponse};
use std::sync::atomic::{AtomicI32, Ordering};
use terrors::OneOf;
use tokio::sync::mpsc;

use crate::errors::{
    ClientTransportError, TransportChannelClosedError, TransportConnectionClosedError,
};

pub(crate) const BUFFER_LENGTH: usize = 16;
static NEXT_REQUEST_ID: AtomicI32 = AtomicI32::new(1);

pub(crate) fn next_request_id() -> i32 {
    NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

pub(crate) struct ClientTransport {
    pub(crate) sender: mpsc::Sender<ClientRequest>,
    pub(crate) receiver: tonic::Streaming<ClientResponse>,
}

impl ClientTransport {
    pub(crate) async fn send(
        &mut self,
        request: ClientRequest,
    ) -> std::result::Result<(), ClientTransportError> {
        self.sender
            .send(request)
            .await
            .map_err(|_| OneOf::new(TransportChannelClosedError))
    }

    pub(crate) async fn recv(
        &mut self,
    ) -> std::result::Result<ClientResponse, ClientTransportError> {
        match self.receiver.message().await {
            Ok(Some(resp)) => Ok(resp),
            Ok(None) => Err(OneOf::new(TransportConnectionClosedError)),
            Err(_) => Err(OneOf::new(TransportConnectionClosedError)),
        }
    }
}
