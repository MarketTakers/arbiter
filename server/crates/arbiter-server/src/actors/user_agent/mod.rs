use crate::{
    actors::{GlobalActors, client::ClientProfile},
    crypto::integrity::Integrable,
    db,
};
use arbiter_crypto::authn;

#[derive(Debug)]
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
    pub fn new(db: db::DatabasePool, actors: GlobalActors) -> Self {
        Self { db, actors }
    }
}

pub mod auth;
pub mod session;

pub use auth::authenticate;
pub use session::UserAgentSession;

use crate::crypto::integrity::hashing::Hashable;

impl Hashable for authn::PublicKey {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        hasher.update(self.to_bytes());
    }
}

impl Hashable for UserAgentCredentials {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        self.pubkey.hash(hasher);
        self.nonce.hash(hasher);
    }
}
