use futures::{Stream, StreamExt};
use tokio::sync::mpsc::{self, error::SendError};
use tonic::{Status, Streaming};


// Abstraction for stream for sans-io capabilities
pub trait Bi<T, U>: Stream<Item = Result<T, Status>> + Send + Sync + 'static {
    type Error;
    fn send(
        &mut self,
        item: Result<U, Status>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}

// Bi-directional stream abstraction for handling gRPC streaming requests and responses
pub struct BiStream<T, U> {
    pub request_stream: Streaming<T>,
    pub response_sender: mpsc::Sender<Result<U, Status>>,
}

impl<T, U> Stream for BiStream<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    type Item = Result<T, Status>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.request_stream.poll_next_unpin(cx)
    }
}

impl<T, U> Bi<T, U> for BiStream<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    type Error = SendError<Result<U, Status>>;

    async fn send(&mut self, item: Result<U, Status>) -> Result<(), Self::Error> {
        self.response_sender.send(item).await
    }
}
