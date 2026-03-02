use kameo::actor::{ActorRef, Spawn};
use miette::Diagnostic;
use thiserror::Error;

use crate::{
    actors::{bootstrap::Bootstrapper, evm::EvmActor, keyholder::KeyHolder, router::MessageRouter},
    db,
};

pub mod bootstrap;
pub mod client;
mod evm;
pub mod keyholder;
pub mod router;
pub mod user_agent;

#[derive(Error, Debug, Diagnostic)]
pub enum SpawnError {
    #[error("Failed to spawn Bootstrapper actor")]
    #[diagnostic(code(SpawnError::Bootstrapper))]
    Bootstrapper(#[from] bootstrap::Error),

    #[error("Failed to spawn KeyHolder actor")]
    #[diagnostic(code(SpawnError::KeyHolder))]
    KeyHolder(#[from] keyholder::Error),
}

/// Long-lived actors that are shared across all connections and handle global state and operations
#[derive(Clone)]
pub struct GlobalActors {
    pub key_holder: ActorRef<KeyHolder>,
    pub bootstrapper: ActorRef<Bootstrapper>,
    pub router: ActorRef<MessageRouter>,
    pub evm: ActorRef<EvmActor>,
}

impl GlobalActors {
    pub async fn spawn(db: db::DatabasePool) -> Result<Self, SpawnError> {
        let key_holder = KeyHolder::spawn(KeyHolder::new(db.clone()).await?);
        Ok(Self {
            bootstrapper: Bootstrapper::spawn(Bootstrapper::new(&db).await?),
            evm: EvmActor::spawn(EvmActor::new(key_holder.clone(), db)),
            key_holder,
            router: MessageRouter::spawn(MessageRouter::default()),
        })
    }
}
