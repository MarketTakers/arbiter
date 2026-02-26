#![forbid(unsafe_code)]
use arbiter_proto::{
    proto::{ClientRequest, ClientResponse, UserAgentRequest, UserAgentResponse},
    transport::{GrpcAdapter, IdentityConverter, ProtocolConverter},
};
use async_trait::async_trait;
use kameo::actor::Spawn;
use tokio_stream::wrappers::ReceiverStream;

use tokio::sync::mpsc;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::{
    actors::user_agent::{UserAgentActor, UserAgentError},
    context::ServerContext,
};

pub mod actors;
pub mod context;
pub mod db;

const DEFAULT_CHANNEL_SIZE: usize = 1000;


/// Converts User Agent domain outbounds into the tonic stream item emitted by
/// the server.
///
/// The conversion is defined at the server boundary so the actor module remains
/// focused on domain semantics and does not depend on tonic status encoding.
struct UserAgentGrpcConverter;

impl ProtocolConverter for UserAgentGrpcConverter {
    type Domain = Result<UserAgentResponse, UserAgentError>;
    type Transport = Result<UserAgentResponse, Status>;

    fn convert(&self, item: Self::Domain) -> Self::Transport {
        match item {
            Ok(message) => Ok(message),
            Err(err) => Err(user_agent_error_status(err)),
        }
    }
}

/// Maps User Agent domain errors to public gRPC transport errors for the
/// `user_agent` streaming endpoint.
fn user_agent_error_status(value: UserAgentError) -> Status {
    match value {
        UserAgentError::MissingRequestPayload | UserAgentError::UnexpectedRequestPayload => {
            Status::invalid_argument("Expected message with payload")
        }
        UserAgentError::InvalidStateForChallengeSolution => {
            Status::invalid_argument("Invalid state for challenge solution")
        }
        UserAgentError::InvalidStateForUnsealEncryptedKey => {
            Status::failed_precondition("Invalid state for unseal encrypted key")
        }
        UserAgentError::InvalidClientPubkeyLength => {
            Status::invalid_argument("client_pubkey must be 32 bytes")
        }
        UserAgentError::InvalidAuthPubkeyLength => {
            Status::invalid_argument("Expected pubkey to have specific length")
        }
        UserAgentError::InvalidAuthPubkeyEncoding => {
            Status::invalid_argument("Failed to convert pubkey to VerifyingKey")
        }
        UserAgentError::InvalidSignatureLength => {
            Status::invalid_argument("Invalid signature length")
        }
        UserAgentError::InvalidBootstrapToken => {
            Status::invalid_argument("Invalid bootstrap token")
        }
        UserAgentError::PublicKeyNotRegistered => {
            Status::unauthenticated("Public key not registered")
        }
        UserAgentError::InvalidChallengeSolution => {
            Status::unauthenticated("Invalid challenge solution")
        }
        UserAgentError::StateTransitionFailed => Status::internal("State machine error"),
        UserAgentError::BootstrapperActorUnreachable => {
            Status::internal("Bootstrap token consumption failed")
        }
        UserAgentError::KeyHolderActorUnreachable => Status::internal("Vault is not available"),
        UserAgentError::DatabasePoolUnavailable => Status::internal("Database pool error"),
        UserAgentError::DatabaseOperationFailed => Status::internal("Database error"),
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

    async fn client(
        &self,
        _request: Request<tonic::Streaming<ClientRequest>>,
    ) -> Result<Response<Self::ClientStream>, Status> {
        todo!()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn user_agent(
        &self,
        request: Request<tonic::Streaming<UserAgentRequest>>,
    ) -> Result<Response<Self::UserAgentStream>, Status> {
        let req_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(DEFAULT_CHANNEL_SIZE);

        let transport = GrpcAdapter::new(
            tx,
            req_stream,
            IdentityConverter::<UserAgentRequest>::new(),
            UserAgentGrpcConverter,
        );
        UserAgentActor::spawn(UserAgentActor::new(self.context.clone(), transport));

        info!(event = "connection established", "grpc.user_agent");

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
