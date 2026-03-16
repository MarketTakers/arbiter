use arbiter_proto::{
    proto::user_agent::{
        SdkClientError as ProtoSdkClientError, UserAgentRequest, UserAgentResponse,
    },
    transport::Bi,
};
use kameo::actor::Spawn as _;
use tracing::{error, info};

use crate::{
    actors::{GlobalActors, user_agent::session::UserAgentSession},
    db::{self},
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum TransportResponseError {
    #[error("Expected message with payload")]
    MissingRequestPayload,
    #[error("Unexpected request payload")]
    UnexpectedRequestPayload,
    #[error("Invalid state for unseal encrypted key")]
    InvalidStateForUnsealEncryptedKey,
    #[error("client_pubkey must be 32 bytes")]
    InvalidClientPubkeyLength,
    #[error("State machine error")]
    StateTransitionFailed,
    #[error("Vault is not available")]
    KeyHolderActorUnreachable,
    #[error("SDK client approve failed: {0:?}")]
    SdkClientApprove(ProtoSdkClientError),
    #[error("SDK client list failed: {0:?}")]
    SdkClientList(ProtoSdkClientError),
    #[error("SDK client revoke failed: {0:?}")]
    SdkClientRevoke(ProtoSdkClientError),
    #[error(transparent)]
    Auth(#[from] auth::Error),
    #[error("Failed registering connection")]
    ConnectionRegistrationFailed,
}

impl TransportResponseError {
    pub fn is_terminal(&self) -> bool {
        !matches!(
            self,
            Self::SdkClientApprove(_) | Self::SdkClientList(_) | Self::SdkClientRevoke(_)
        )
    }
}

pub type Transport =
    Box<dyn Bi<UserAgentRequest, Result<UserAgentResponse, TransportResponseError>> + Send>;

pub struct UserAgentConnection {
    db: db::DatabasePool,
    actors: GlobalActors,
    transport: Transport,
}

impl UserAgentConnection {
    pub fn new(db: db::DatabasePool, actors: GlobalActors, transport: Transport) -> Self {
        Self {
            db,
            actors,
            transport,
        }
    }
}

pub mod auth;
pub mod session;

pub async fn connect_user_agent(props: UserAgentConnection) {
    match auth::authenticate_and_create(props).await {
        Ok(session) => {
            UserAgentSession::spawn(session);
            info!("User authenticated, session started");
        }
        Err(err) => {
            error!(?err, "Authentication failed, closing connection");
        }
    }
}
