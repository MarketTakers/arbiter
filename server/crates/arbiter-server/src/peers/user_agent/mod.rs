use crate::{
    actors::GlobalActors, crypto::integrity::Integrable, db, peers::client::ClientProfile,
};
use arbiter_crypto::authn;

pub mod auth;
pub mod session;
pub mod vault_gate;


#[derive(Debug, Clone, Hash)]
pub struct Credentials {
    pub id: i32,
    pub pubkey: authn::PublicKey,
}
impl Hashable for Credentials {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        self.id.hash(hasher);
        self.pubkey.hash(hasher);
    }
}

#[derive(Debug, Clone)]
pub struct AuthCredentials {
    pub creds: Credentials,
    // denotes new nonce, not current
    pub new_nonce: i32,
}

impl Hashable for authn::PublicKey {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        hasher.update(self.to_bytes());
    }
}

impl Hashable for AuthCredentials {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        self.creds.hash(hasher);
        self.new_nonce.hash(hasher);
    }
}


impl Integrable for AuthCredentials {
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



pub use auth::authenticate;
pub use session::UserAgentSession;

use crate::crypto::integrity::hashing::Hashable;