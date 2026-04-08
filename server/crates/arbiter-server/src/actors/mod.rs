use kameo::actor::{ActorRef, Spawn};
use kameo_actors::{DeliveryStrategy, message_bus::MessageBus};
use thiserror::Error;

use crate::{
    actors::{
        bootstrap::Bootstrapper, evm::EvmActor, flow_coordinator::FlowCoordinator, vault::Vault,
    },
    db,
};

pub mod bootstrap;
pub mod evm;
pub mod flow_coordinator;
pub mod vault;
pub mod useragent_registry;

#[derive(Error, Debug)]
pub enum SpawnError {
    #[error("Failed to spawn Bootstrapper actor")]
    Bootstrapper(#[from] bootstrap::Error),

    #[error("Failed to spawn Vault actor")]
    Vault(#[from] vault::Error),
}

/// Long-lived actors that are shared across all connections and handle global state and operations
#[derive(Clone)]
pub struct GlobalActors {
    pub vault: ActorRef<Vault>,
    pub bootstrapper: ActorRef<Bootstrapper>,
    pub flow_coordinator: ActorRef<FlowCoordinator>,
    pub evm: ActorRef<EvmActor>,
    pub events: ActorRef<MessageBus>,
}

impl GlobalActors {
    pub fn spawn_message_bus() -> ActorRef<MessageBus> {
        MessageBus::spawn(MessageBus::new(DeliveryStrategy::Guaranteed))
    }

    pub async fn spawn(db: db::DatabasePool) -> Result<Self, SpawnError> {
        let message_bus = Self::spawn_message_bus();
        let key_holder = Vault::spawn(Vault::new(db.clone(), message_bus.clone()).await?);
        Ok(Self {
            bootstrapper: Bootstrapper::spawn(Bootstrapper::new(&db).await?),
            evm: EvmActor::spawn(EvmActor::new(key_holder.clone(), db)),
            vault: key_holder,
            flow_coordinator: FlowCoordinator::spawn(FlowCoordinator::default()),
            events: message_bus,
        })
    }
}
