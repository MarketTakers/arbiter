#![forbid(unsafe_code)]
use arbiter_proto::{
    proto::{
        client::{ClientRequest, ClientResponse},
        user_agent::{UserAgentRequest, UserAgentResponse},
    },
    transport::{IdentityRecvConverter, SendConverter, grpc},
};
use async_trait::async_trait;
use tokio_stream::wrappers::ReceiverStream;

use tokio::sync::mpsc;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::{
    actors::{
        client::{self, ClientConnection as ClientConnectionProps, ClientError, connect_client},
        user_agent::{self, TransportResponseError, UserAgentConnection, connect_user_agent},
    },
    context::ServerContext,
};

pub mod actors;
pub mod context;
pub mod db;
pub mod evm;

const DEFAULT_CHANNEL_SIZE: usize = 1000;

struct UserAgentGrpcSender;

impl SendConverter for UserAgentGrpcSender {
    type Input = Result<UserAgentResponse, TransportResponseError>;
    type Output = Result<UserAgentResponse, Status>;

    fn convert(&self, item: Self::Input) -> Self::Output {
        match item {
            Ok(message) => Ok(message),
            Err(err) => Err(user_agent_error_status(err)),
        }
    }
}

struct ClientGrpcSender;

impl SendConverter for ClientGrpcSender {
    type Input = Result<ClientResponse, ClientError>;
    type Output = Result<ClientResponse, Status>;

    fn convert(&self, item: Self::Input) -> Self::Output {
        match item {
            Ok(message) => Ok(message),
            Err(err) => Err(client_error_status(err)),
        }
    }
}

fn client_error_status(value: ClientError) -> Status {
    match value {
        ClientError::MissingRequestPayload | ClientError::UnexpectedRequestPayload => {
            Status::invalid_argument("Expected message with payload")
        }
        ClientError::StateTransitionFailed => Status::internal("State machine error"),
        ClientError::Auth(ref err) => client_auth_error_status(err),
        ClientError::ConnectionRegistrationFailed => {
            Status::internal("Connection registration failed")
        }
    }
}

fn client_auth_error_status(value: &client::auth::Error) -> Status {
    use client::auth::Error;
    match value {
        Error::UnexpectedMessagePayload | Error::InvalidClientPubkeyLength => {
            Status::invalid_argument(value.to_string())
        }
        Error::InvalidAuthPubkeyEncoding => {
            Status::invalid_argument("Failed to convert pubkey to VerifyingKey")
        }
        Error::InvalidChallengeSolution => Status::unauthenticated(value.to_string()),
        Error::ApproveError(_) => Status::permission_denied(value.to_string()),
        Error::Transport => Status::internal("Transport error"),
        Error::DatabasePoolUnavailable => Status::internal("Database pool error"),
        Error::DatabaseOperationFailed => Status::internal("Database error"),
        Error::InternalError => Status::internal("Internal error"),
    }
}

fn user_agent_error_status(value: TransportResponseError) -> Status {
    match value {
        TransportResponseError::MissingRequestPayload
        | TransportResponseError::UnexpectedRequestPayload => {
            Status::invalid_argument("Expected message with payload")
        }
        TransportResponseError::InvalidStateForUnsealEncryptedKey => {
            Status::failed_precondition("Invalid state for unseal encrypted key")
        }
        TransportResponseError::InvalidClientPubkeyLength => {
            Status::invalid_argument("client_pubkey must be 32 bytes")
        }
        TransportResponseError::StateTransitionFailed => Status::internal("State machine error"),
        TransportResponseError::KeyHolderActorUnreachable => {
            Status::internal("Vault is not available")
        }
        TransportResponseError::Auth(ref err) => auth_error_status(err),
        TransportResponseError::ConnectionRegistrationFailed => {
            Status::internal("Failed registering connection")
        }
    }
}

fn auth_error_status(value: &user_agent::auth::Error) -> Status {
    use user_agent::auth::Error;
    match value {
        Error::UnexpectedMessagePayload | Error::InvalidClientPubkeyLength => {
            Status::invalid_argument(value.to_string())
        }
        Error::InvalidAuthPubkeyEncoding => {
            Status::invalid_argument("Failed to convert pubkey to VerifyingKey")
        }
        Error::PublicKeyNotRegistered | Error::InvalidChallengeSolution => {
            Status::unauthenticated(value.to_string())
        }
        Error::InvalidBootstrapToken => Status::invalid_argument("Invalid bootstrap token"),
        Error::Transport => Status::internal("Transport error"),
        Error::BootstrapperActorUnreachable => {
            Status::internal("Bootstrap token consumption failed")
        }
        Error::DatabasePoolUnavailable => Status::internal("Database pool error"),
        Error::DatabaseOperationFailed => Status::internal("Database error"),
    }
}

pub struct Server {
    context: ServerContext,
}

impl Server {
    pub fn new(context: ServerContext) -> Self {
        Self { context }
    }
}

#[async_trait]
impl arbiter_proto::proto::arbiter_service_server::ArbiterService for Server {
    type UserAgentStream = ReceiverStream<Result<UserAgentResponse, Status>>;
    type ClientStream = ReceiverStream<Result<ClientResponse, Status>>;

    #[tracing::instrument(level = "debug", skip(self))]
    async fn client(
        &self,
        request: Request<tonic::Streaming<ClientRequest>>,
    ) -> Result<Response<Self::ClientStream>, Status> {
        let req_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(DEFAULT_CHANNEL_SIZE);

        let transport = grpc::GrpcAdapter::new(
            tx,
            req_stream,
            IdentityRecvConverter::<ClientRequest>::new(),
            ClientGrpcSender,
        );
        let props = ClientConnectionProps::new(
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

        let transport = grpc::GrpcAdapter::new(
            tx,
            req_stream,
            IdentityRecvConverter::<UserAgentRequest>::new(),
            UserAgentGrpcSender,
        );
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
