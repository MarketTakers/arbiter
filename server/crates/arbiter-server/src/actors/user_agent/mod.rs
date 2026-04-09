use crate::{
    actors::{GlobalActors, client::ClientProfile},
    crypto::integrity::Integrable,
    db,
};
use arbiter_crypto::authn;

#[derive(Debug, arbiter_macros::Hashable)]
pub struct UserAgentCredentials {
    pub pubkey: authn::PublicKey,
    pub nonce: i32,
}

impl Integrable for UserAgentCredentials {
    const KIND: &'static str = "useragent_credentials";
}

// Messages, sent by user agent to connection client without having a request
#[derive(Debug)]
pub enum OutOfBand {
    ClientConnectionRequest { profile: ClientProfile },
    ClientConnectionCancel { pubkey: authn::PublicKey },
}

pub struct UserAgentConnection {
    pub(crate) db: db::DatabasePool,
    pub(crate) actors: GlobalActors,
}

impl UserAgentConnection {
    pub const fn new(db: db::DatabasePool, actors: GlobalActors) -> Self {
        Self { db, actors }
    }
}

pub mod auth;
pub mod session;

pub use auth::authenticate;
pub use session::UserAgentSession;
