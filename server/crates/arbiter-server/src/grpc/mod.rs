use crate::peers::{client::ClientConnection, user_agent::UserAgentConnection};
use arbiter_proto::{
    proto::{
        client::{ClientRequest, ClientResponse},
        user_agent::{UserAgentRequest, UserAgentResponse},
    },
    transport::grpc::GrpcBi,
};

use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, async_trait};
use tracing::info;

mod request_tracker;

pub mod client;
pub mod user_agent;

mod common;

pub trait Convert {
    type Output;

    fn convert(self) -> Self::Output;
}

pub trait TryConvert {
    type Output;
    type Error;

    fn try_convert(self) -> Result<Self::Output, Self::Error>;
}

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
        let (bi, rx) = GrpcBi::from_bi_stream(req_stream);
        let props = ClientConnection::new(self.context.db.clone(), self.context.actors.clone());
        tokio::spawn(client::start(props, bi));

        info!(event = "connection established", "grpc.client");

        Ok(Response::new(rx))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn user_agent(
        &self,
        request: Request<tonic::Streaming<UserAgentRequest>>,
    ) -> Result<Response<Self::UserAgentStream>, Status> {
        let req_stream = request.into_inner();

        let (bi, rx) = GrpcBi::from_bi_stream(req_stream);

        tokio::spawn(user_agent::start(
            UserAgentConnection {
                db: self.context.db.clone(),
                actors: self.context.actors.clone(),
            },
            bi,
        ));

        info!(event = "connection established", "grpc.user_agent");

        Ok(Response::new(rx))
    }
}
