use arbiter_proto::{
    proto::client::{ClientRequest, ClientResponse},
    transport::Bi,
};
use kameo::actor::Spawn;
use tracing::{error, info};

use crate::{actors::client::session::ClientSession, db};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ClientError {
    #[error("Expected message with payload")]
    MissingRequestPayload,
    #[error("Unexpected request payload")]
    UnexpectedRequestPayload,
    #[error("State machine error")]
    StateTransitionFailed,
    #[error(transparent)]
    Auth(#[from] auth::Error),
}

pub type Transport = Box<dyn Bi<ClientRequest, Result<ClientResponse, ClientError>> + Send>;

pub struct ConnectionProps {
    pub(crate) db: db::DatabasePool,
    pub(crate) transport: Transport,
}

impl ConnectionProps {
    pub fn new(db: db::DatabasePool, transport: Transport) -> Self {
        Self { db, transport }
    }
}

pub mod auth;
pub mod session;

pub async fn connect_client(props: ConnectionProps) {
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
