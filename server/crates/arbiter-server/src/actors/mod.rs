use crate::{
    actors::{
        bootstrap::Bootstrapper, evm::EvmActor, flow_coordinator::FlowCoordinator,
        operator_registry::OperatorRegistry, vault::Vault, vault_coordinator::VaultCoordinator,
    },
    db::{
        self,
        custody::{CustodyStore, DieselCustodyStore},
    },
};

use std::sync::Arc;

use kameo::actor::{ActorRef, Spawn};
use kameo_actors::{DeliveryStrategy, message_bus::MessageBus};
use thiserror::Error;

pub mod bootstrap;
pub mod evm;
pub mod flow_coordinator;
pub mod operator_registry;
pub mod vault;
pub mod vault_coordinator;

#[derive(Error, Debug)]
pub enum SpawnError {
    #[error("Failed to spawn Bootstrapper actor")]
    Bootstrapper(#[from] bootstrap::Error),
    #[error("Failed to spawn Vault actor")]
    Vault(#[from] vault::Error),
    #[error("Failed to spawn VaultCoordinator actor")]
    VaultCoordinator(#[from] vault_coordinator::Error),
}

#[derive(Clone)]
pub struct GlobalActors {
    pub vault: ActorRef<Vault>,
    pub vault_coordinator: ActorRef<VaultCoordinator>,
    pub bootstrapper: ActorRef<Bootstrapper>,
    pub flow_coordinator: ActorRef<FlowCoordinator>,
    pub operator_registry: ActorRef<OperatorRegistry>,
    pub evm: ActorRef<EvmActor>,
    pub events: ActorRef<MessageBus>,
}

impl GlobalActors {
    pub fn spawn_message_bus() -> ActorRef<MessageBus> {
        MessageBus::spawn(MessageBus::new(DeliveryStrategy::Guaranteed))
    }

    pub async fn spawn(db: db::DatabasePool) -> Result<Self, SpawnError> {
        let events = Self::spawn_message_bus();
        let custody: Arc<dyn CustodyStore> = Arc::new(DieselCustodyStore);
        let vault =
            Vault::spawn(Vault::new(db.clone(), events.clone(), Arc::clone(&custody)).await?);
        let bootstrapper = Bootstrapper::spawn(Bootstrapper::new(&db, events.clone()).await?);
        let vault_coordinator =
            VaultCoordinator::spawn(VaultCoordinator::new(db.clone(), vault.clone(), custody));
        let operator_registry = OperatorRegistry::spawn(OperatorRegistry::default());
        Ok(Self {
            bootstrapper,
            evm: EvmActor::spawn(EvmActor::new(vault.clone(), db.clone())),
            vault,
            vault_coordinator,
            flow_coordinator: FlowCoordinator::spawn(FlowCoordinator::new(
                operator_registry.clone(),
            )),
            operator_registry,
            events,
        })
    }
}
