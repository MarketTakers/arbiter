use arbiter_proto::transport::Bi;
use kameo::actor::Spawn;
use tracing::{error, info};

use crate::{
    actors::{GlobalActors, client::session::ClientSession},
    db,
};

pub struct ClientConnection {
    pub(crate) db: db::DatabasePool,
    pub(crate) actors: GlobalActors,
    pub(crate) client_id: i32,
}

impl ClientConnection {
    pub fn new(db: db::DatabasePool, actors: GlobalActors) -> Self {
        Self {
            db,
            actors,
            client_id: 0,
        }
    }
}

pub mod auth;
pub mod session;

pub async fn connect_client<T>(mut props: ClientConnection, transport: &mut T)
where
    T: Bi<auth::Inbound, Result<auth::Outbound, auth::Error>> + Send + ?Sized,
{
    match auth::authenticate(&mut props, transport).await {
        Ok(authenticated) => {
            props.client_id = authenticated.client_id;
            ClientSession::spawn(ClientSession::new(props));
            info!("Client authenticated, session started");
        }
        Err(err) => {
            let _ = transport.send(Err(err.clone())).await;
            error!(?err, "Authentication failed, closing connection");
        }
    }
}
