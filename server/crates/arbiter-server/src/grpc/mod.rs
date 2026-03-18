
use arbiter_proto::proto::{
    client::{ClientRequest, ClientResponse},
    user_agent::{UserAgentRequest, UserAgentResponse},
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, async_trait};
use tracing::info;

use crate::{
    DEFAULT_CHANNEL_SIZE,
    actors::{client::{ClientConnection, connect_client}, user_agent::{UserAgentConnection, connect_user_agent}},
};

pub mod client;
pub mod user_agent;

#[async_trait]
impl arbiter_proto::proto::arbiter_service_server::ArbiterService for super::Server {
    type UserAgentStream = ReceiverStream<Result<UserAgentResponse, Status>>;
    type ClientStream = ReceiverStream<Result<ClientResponse, Status>>;

    #[tracing::instrument(level = "debug", skip(self))]
    async fn client(
        &self,
        request: Request<tonic::Streaming<ClientRequest>>,
    ) -> Result<Response<Self::ClientStream>, Status> {
        let req_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(DEFAULT_CHANNEL_SIZE);

        let transport = client::GrpcTransport::new(tx, req_stream);
        let props = ClientConnection::new(
            self.context.db.clone(),
            Box::new(transport),
            self.context.actors.clone(),
        );
        tokio::spawn(connect_client(props));

        info!(event = "connection established", "grpc.client");

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn user_agent(
        &self,
        request: Request<tonic::Streaming<UserAgentRequest>>,
    ) -> Result<Response<Self::UserAgentStream>, Status> {
        let req_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(DEFAULT_CHANNEL_SIZE);

        let transport = user_agent::GrpcTransport::new(tx, req_stream);
        let props = UserAgentConnection::new(
            self.context.db.clone(),
            self.context.actors.clone(),
            Box::new(transport),
        );
        tokio::spawn(connect_user_agent(props));

        info!(event = "connection established", "grpc.user_agent");

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
