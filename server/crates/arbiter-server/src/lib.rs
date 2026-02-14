#![allow(unused)]

use std::sync::Arc;

use arbiter_proto::{
    proto::{ClientRequest, ClientResponse, UserAgentRequest, UserAgentResponse},
    transport::BiStream,
};
use async_trait::async_trait;
use tokio_stream::wrappers::ReceiverStream;

use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

use crate::{
    actors::{client::handle_client, user_agent::handle_user_agent},
    context::ServerContext,
};

pub mod actors;
mod context;
mod db;
mod errors;

const DEFAULT_CHANNEL_SIZE: usize = 1000;

pub struct Server {
    context: ServerContext,
}

#[async_trait]
impl arbiter_proto::proto::arbiter_service_server::ArbiterService for Server {
    type UserAgentStream = ReceiverStream<Result<UserAgentResponse, Status>>;
    type ClientStream = ReceiverStream<Result<ClientResponse, Status>>;

    async fn client(
        &self,
        request: Request<tonic::Streaming<ClientRequest>>,
    ) -> Result<Response<Self::ClientStream>, Status> {
        let req_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(DEFAULT_CHANNEL_SIZE);
        tokio::spawn(handle_client(
            self.context.clone(),
            BiStream {
                request_stream: req_stream,
                response_sender: tx,
            },
        ));

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn user_agent(
        &self,
        request: Request<tonic::Streaming<UserAgentRequest>>,
    ) -> Result<Response<Self::UserAgentStream>, Status> {
        let req_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(DEFAULT_CHANNEL_SIZE);
        tokio::spawn(handle_user_agent(self.context.clone(), req_stream, tx));
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
