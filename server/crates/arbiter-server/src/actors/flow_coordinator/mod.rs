use crate::{
    actors::{
        flow_coordinator::client_connect_approval::ClientApprovalController,
        operator_registry::{GetConnected, OperatorRegistry},
    },
    peers::client::{ClientProfile, session::ClientSession},
};

use kameo::{
    Actor,
    actor::{ActorId, ActorRef, Spawn},
    messages,
    prelude::{ActorStopReason, Context, WeakActorRef},
    reply::DelegatedReply,
};
use std::{collections::HashMap, ops::ControlFlow};
use tracing::info;

pub mod client_connect_approval;

pub struct FlowCoordinator {
    pub clients: HashMap<ActorId, ActorRef<ClientSession>>,
    /// Maps DB `client_id` → `ActorId` for fast connected-client lookup.
    client_ids: HashMap<i32, ActorId>,
    operator_registry: ActorRef<OperatorRegistry>,
}

impl FlowCoordinator {
    pub fn new(operator_registry: ActorRef<OperatorRegistry>) -> Self {
        Self {
            clients: HashMap::default(),
            client_ids: HashMap::default(),
            operator_registry,
        }
    }
}

impl Actor for FlowCoordinator {
    type Args = Self;

    type Error = ();

    async fn on_start(args: Self::Args, _: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(args)
    }

    async fn on_link_died(
        &mut self,
        _: WeakActorRef<Self>,
        id: ActorId,
        _: ActorStopReason,
    ) -> Result<ControlFlow<ActorStopReason>, Self::Error> {
        if self.clients.remove(&id).is_some() {
            self.client_ids.retain(|_, actor_id| *actor_id != id);
            info!(
                ?id,
                actor = "FlowCoordinator",
                event = "client.disconnected"
            );
        } else {
            info!(
                ?id,
                actor = "FlowCoordinator",
                event = "unknown.actor.disconnected"
            );
        }
        Ok(ControlFlow::Continue(()))
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq, Hash)]
pub enum ApprovalError {
    #[error("No operators connected")]
    NoOperatorsConnected,
}

#[messages]
impl FlowCoordinator {
    #[message(ctx)]
    pub async fn register_client(
        &mut self,
        client_id: i32,
        actor: ActorRef<ClientSession>,
        ctx: &mut Context<Self, ()>,
    ) {
        info!(id = %actor.id(), client_id, actor = "FlowCoordinator", event = "client.connected");
        ctx.actor_ref().link(&actor).await;
        self.client_ids.insert(client_id, actor.id());
        self.clients.insert(actor.id(), actor);
    }

    #[message]
    pub fn is_client_connected(&self, client_id: i32) -> bool {
        self.client_ids.contains_key(&client_id)
    }

    /// Returns the DB `client_ids` of all currently connected SDK clients.
    /// Used by operator sessions on startup to seed their approved-client set.
    #[message]
    pub fn get_connected_client_ids(&self) -> Vec<i32> {
        self.client_ids.keys().copied().collect()
    }

    #[message(ctx)]
    pub async fn request_client_approval(
        &mut self,
        client: ClientProfile,
        ctx: &mut Context<Self, DelegatedReply<Result<bool, ApprovalError>>>,
    ) -> DelegatedReply<Result<bool, ApprovalError>> {
        let (reply, Some(reply_sender)) = ctx.reply_sender() else {
            unreachable!("Expected `request_client_approval` to have callback channel");
        };

        let Ok(refs) = self.operator_registry.ask(GetConnected).await else {
            reply_sender.send(Err(ApprovalError::NoOperatorsConnected));
            return reply;
        };

        if refs.is_empty() {
            reply_sender.send(Err(ApprovalError::NoOperatorsConnected));
            return reply;
        }

        ClientApprovalController::spawn(client_connect_approval::Args {
            client,
            operators: refs,
            reply: reply_sender,
        });

        reply
    }
}
