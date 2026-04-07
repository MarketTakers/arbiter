use arbiter_proto::{ClientMetadata, transport::Bi};
use kameo::actor::Spawn;
use tracing::{error, info};

use crate::{
    actors::{GlobalActors, client::session::ClientSession},
    crypto::authn,
    crypto::integrity::{Integrable, hashing::Hashable},
    db,
};

#[derive(Debug, Clone)]
pub struct ClientProfile {
    pub pubkey: authn::PublicKey,
    pub metadata: ClientMetadata,
}

pub struct ClientCredentials {
    pub pubkey: authn::PublicKey,
    pub nonce: i32,
}

impl Integrable for ClientCredentials {
    const KIND: &'static str = "client_credentials";
}

impl Hashable for ClientCredentials {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        hasher.update(self.pubkey.to_bytes());
        self.nonce.hash(hasher);
    }
}

pub struct ClientConnection {
    pub(crate) db: db::DatabasePool,
    pub(crate) actors: GlobalActors,
}

impl ClientConnection {
    pub fn new(db: db::DatabasePool, actors: GlobalActors) -> Self {
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
    println!(
        "authenticate future size: {}",
        std::mem::size_of_val(&fut)
    );
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
