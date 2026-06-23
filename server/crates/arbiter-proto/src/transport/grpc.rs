use super::{Bi, Receiver, Sender};

use async_trait::async_trait;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct GrpcSender<Outbound> {
    tx: mpsc::Sender<Result<Outbound, tonic::Status>>,
}

#[async_trait]
impl<Outbound> Sender<Result<Outbound, tonic::Status>> for GrpcSender<Outbound>
where
    Outbound: Send + Sync + 'static,
{
    async fn send(&mut self, item: Result<Outbound, tonic::Status>) -> Result<(), super::Error> {
        self.tx
            .send(item)
            .await
            .map_err(|_| super::Error::ChannelClosed)
    }
}

pub struct GrpcReceiver<Inbound> {
    rx: tonic::Streaming<Inbound>,
}
#[async_trait]
impl<Inbound> Receiver<Result<Inbound, tonic::Status>> for GrpcReceiver<Inbound>
where
    Inbound: Send + Sync + 'static,
{
    async fn recv(&mut self) -> Option<Result<Inbound, tonic::Status>> {
        self.rx.next().await
    }
}

pub struct GrpcBi<Inbound, Outbound> {
    sender: GrpcSender<Outbound>,
    receiver: GrpcReceiver<Inbound>,
}

impl<Inbound, Outbound> GrpcBi<Inbound, Outbound>
where
    Inbound: Send + Sync + 'static,
    Outbound: Send + Sync + 'static,
{
    pub fn from_bi_stream(
        receiver: tonic::Streaming<Inbound>,
    ) -> (Self, ReceiverStream<Result<Outbound, tonic::Status>>) {
        let (tx, rx) = mpsc::channel(10);
        let sender = GrpcSender { tx };
        let receiver = GrpcReceiver { rx: receiver };
        let bi = GrpcBi { sender, receiver };
        (bi, ReceiverStream::new(rx))
    }
}

#[async_trait]
impl<Inbound, Outbound> Sender<Result<Outbound, tonic::Status>> for GrpcBi<Inbound, Outbound>
where
    Inbound: Send + Sync + 'static,
    Outbound: Send + Sync + 'static,
{
    async fn send(&mut self, item: Result<Outbound, tonic::Status>) -> Result<(), super::Error> {
        self.sender.send(item).await
    }
}

#[async_trait]
impl<Inbound, Outbound> Receiver<Result<Inbound, tonic::Status>> for GrpcBi<Inbound, Outbound>
where
    Inbound: Send + Sync + 'static,
    Outbound: Send + Sync + 'static,
{
    async fn recv(&mut self) -> Option<Result<Inbound, tonic::Status>> {
        self.receiver.recv().await
    }
}

impl<Inbound, Outbound> Bi<Result<Inbound, tonic::Status>, Result<Outbound, tonic::Status>>
    for GrpcBi<Inbound, Outbound>
where
    Inbound: Send + Sync + 'static,
    Outbound: Send + Sync + 'static,
{
}

impl<Inbound, Outbound>
    super::SplittableBi<Result<Inbound, tonic::Status>, Result<Outbound, tonic::Status>>
    for GrpcBi<Inbound, Outbound>
where
    Inbound: Send + Sync + 'static,
    Outbound: Send + Sync + 'static,
{
    type Sender = GrpcSender<Outbound>;
    type Receiver = GrpcReceiver<Inbound>;

    fn split(self) -> (Self::Sender, Self::Receiver) {
        (self.sender, self.receiver)
    }

    fn from_parts(sender: Self::Sender, receiver: Self::Receiver) -> Self {
        GrpcBi { sender, receiver }
    }
}
