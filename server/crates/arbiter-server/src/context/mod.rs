use crate::{
    actors::GlobalActors,
    context::tls::TlsManager,
    db::{self},
};

use std::sync::Arc;
use thiserror::Error;

pub mod tls;

#[derive(Error, Debug)]
pub enum InitError {
    #[error("Database setup failed: {0}")]
    DatabaseSetup(#[from] db::DatabaseSetupError),

    #[error("Connection acquire failed: {0}")]
    DatabasePool(#[from] db::PoolError),

    #[error("Database query error: {0}")]
    DatabaseQuery(#[from] diesel::result::Error),

    #[error("TLS initialization failed: {0}")]
    Tls(#[from] tls::InitError),

    #[error("Actor spawn failed: {0}")]
    ActorSpawn(#[from] crate::actors::SpawnError),

    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct __ServerContextInner {
    pub db: db::DatabasePool,
    pub tls: TlsManager,
    pub actors: GlobalActors,
}
#[derive(Clone)]
pub struct ServerContext(Arc<__ServerContextInner>);

impl std::ops::Deref for ServerContext {
    type Target = __ServerContextInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ServerContext {
    pub async fn new(db: db::DatabasePool) -> Result<Self, InitError> {
        Ok(Self(Arc::new(__ServerContextInner {
            actors: GlobalActors::spawn(db.clone()).await?,
            tls: TlsManager::new(db.clone()).await?,
            db,
        })))
    }
}
