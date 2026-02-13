#![allow(unused)]

use std::sync::Arc;

use tracing::error;

use arbiter_proto::{
    proto::{
        ClientRequest, ClientResponse, UserAgentRequest, UserAgentResponse,
        auth::{
            self, AuthChallengeRequest, ClientMessage, client_message::Payload as ClientAuthPayload,
        },
        user_agent_request::Payload as UserAgentRequestPayload,
        user_agent_request::*,
    },
    transport::BiStream,
};
use async_trait::async_trait;
use futures::StreamExt;
use kameo::actor::Spawn;
use tokio_stream::wrappers::ReceiverStream;

use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

use crate::{
    actors::{
        client::handle_client,
        user_agent::{self, UserAgentActor},
    },
    context::ServerContext,
};

pub mod actors;
mod context;
mod db;

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
        let mut req_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(DEFAULT_CHANNEL_SIZE);

        let actor = UserAgentActor::spawn(UserAgentActor::new(self.context.clone(), tx.clone()));

        tokio::task::spawn(async move {
            while let Some(Ok(req)) = req_stream.next().await
                && actor.is_alive()
            {
                let Some(msg) = req.payload else {
                    error!(actor = "useragent", "Received message with no payload");
                    actor.kill();
                    tx.send(Err(Status::invalid_argument(
                        "Expected message with payload",
                    )))
                    .await;
                    return;
                };

                let UserAgentRequestPayload::AuthMessage(ClientMessage {
                    payload: Some(client_message),
                }) = msg
                else {
                    error!(
                        actor = "useragent",
                        "Received unexpected message type during authentication"
                    );
                    actor.kill();
                    tx.send(Err(Status::invalid_argument(
                        "Expected AuthMessage with ClientMessage payload",
                    )))
                    .await;
                    return;
                };

                match client_message {
                    ClientAuthPayload::AuthChallengeRequest(req) => {}
                    ClientAuthPayload::AuthChallengeSolution(_auth_challenge_solution) => todo!(),
                    _ => {
                        error!(actor = "useragent", "Received unexpected message type");
                        actor.kill();
                        tx.send(Err(Status::invalid_argument(
                            "Expected AuthMessage with ClientMessage payload",
                        )))
                        .await;
                        return;
                    }
                }
                todo!()
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
