use arbiter_proto::transport::Bi;
use kameo::actor::Spawn;
use tracing::{error, info};

use crate::{
    actors::{GlobalActors, client::session::ClientSession},
    db,
};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ClientError {
    #[error("Expected message with payload")]
    MissingRequestPayload,
    #[error("Unexpected request payload")]
    UnexpectedRequestPayload,
    #[error("State machine error")]
    StateTransitionFailed,
    #[error("Connection registration failed")]
    ConnectionRegistrationFailed,
    #[error(transparent)]
    Auth(#[from] auth::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectErrorCode {
    Unknown,
    ApprovalDenied,
    NoUserAgentsOnline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    AuthChallengeRequest { pubkey: Vec<u8> },
    AuthChallengeSolution { signature: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    AuthChallenge { pubkey: Vec<u8>, nonce: i32 },
    AuthOk,
    ClientConnectError { code: ConnectErrorCode },
}

pub type Transport = Box<dyn Bi<Request, Result<Response, ClientError>> + Send>;

pub struct ClientConnection {
    pub(crate) db: db::DatabasePool,
    pub(crate) transport: Transport,
    pub(crate) actors: GlobalActors,
}

impl ClientConnection {
    pub fn new(db: db::DatabasePool, transport: Transport, actors: GlobalActors) -> Self {
        Self {
            db,
            transport,
            actors,
        }
    }
}

pub mod auth;
pub mod session;

pub async fn connect_client(props: ClientConnection) {
    match auth::authenticate_and_create(props).await {
        Ok(session) => {
            ClientSession::spawn(session);
            info!("Client authenticated, session started");
        }
        Err(err) => {
            error!(?err, "Authentication failed, closing connection");
        }
    }
}
