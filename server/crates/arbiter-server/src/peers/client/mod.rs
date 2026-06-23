use crate::{
    actors::GlobalActors, crypto::integrity::Integrable, db, peers::client::session::ClientSession,
};
use arbiter_crypto::authn;
use arbiter_macros::Hashable;
use arbiter_proto::{ClientMetadata, transport::Bi};

use kameo::actor::Spawn;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct ClientProfile {
    pub pubkey: authn::PublicKey,
    pub metadata: ClientMetadata,
}

#[derive(Hashable)]
pub struct ClientCredentials {
    pub pubkey: authn::PublicKey,
}

impl Integrable for ClientCredentials {
    const KIND: &'static str = "client_credentials";
}

pub struct ClientConnection {
    pub(crate) db: db::DatabasePool,
    pub(crate) actors: GlobalActors,
}

impl ClientConnection {
    pub const fn new(db: db::DatabasePool, actors: GlobalActors) -> Self {
        Self { db, actors }
    }
}

pub mod auth;
pub mod session;

pub async fn connect_client<T>(mut props: ClientConnection, transport: &mut T)
where
    T: Bi<auth::Inbound, Result<auth::Outbound, auth::Error>> + Send + ?Sized,
{
    let fut = auth::authenticate(&mut props, transport);
    println!("authenticate future size: {}", size_of_val(&fut));
    match fut.await {
        Ok(client_id) => {
            ClientSession::spawn(ClientSession::new(props, client_id));
            info!("Client authenticated, session started");
        }
        Err(err) => {
            let _ = transport.send(Err(err.clone())).await;
            error!(?err, "Authentication failed, closing connection");
        }
    }
}
