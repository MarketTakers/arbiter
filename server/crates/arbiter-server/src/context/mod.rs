use std::sync::Arc;

use miette::Diagnostic;
use thiserror::Error;

use crate::{
    actors::GlobalActors,
    context::tls::TlsManager,
    db::{self},
};

pub mod tls;

#[derive(Error, Debug, Diagnostic)]
pub enum InitError {
    #[error("Database setup failed: {0}")]
    #[diagnostic(code(arbiter_server::init::database_setup))]
    DatabaseSetup(#[from] db::DatabaseSetupError),

    #[error("Connection acquire failed: {0}")]
    #[diagnostic(code(arbiter_server::init::database_pool))]
    DatabasePool(#[from] db::PoolError),

    #[error("Database query error: {0}")]
    #[diagnostic(code(arbiter_server::init::database_query))]
    DatabaseQuery(#[from] diesel::result::Error),

    #[error("TLS initialization failed: {0}")]
    #[diagnostic(code(arbiter_server::init::tls_init))]
    Tls(#[from] tls::InitError),

    #[error("Actor spawn failed: {0}")]
    #[diagnostic(code(arbiter_server::init::actor_spawn))]
    ActorSpawn(#[from] crate::actors::SpawnError),

    #[error("I/O Error: {0}")]
    #[diagnostic(code(arbiter_server::init::io))]
    Io(#[from] std::io::Error),
}

pub struct _ServerContextInner {
    pub db: db::DatabasePool,
    pub tls: TlsManager,
    pub actors: GlobalActors,
}
#[derive(Clone)]
pub struct ServerContext(Arc<_ServerContextInner>);

impl std::ops::Deref for ServerContext {
    type Target = _ServerContextInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ServerContext {
    pub async fn new(db: db::DatabasePool) -> Result<Self, InitError> {
        Ok(Self(Arc::new(_ServerContextInner {
            actors: GlobalActors::spawn(db.clone()).await?,
            tls: TlsManager::new(db.clone()).await?,
            db,
        })))
    }
}
