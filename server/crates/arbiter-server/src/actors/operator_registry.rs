use crate::peers::operator::OperatorSession;

use kameo::{
    Actor,
    actor::{ActorId, ActorRef},
    error::Infallible,
    messages,
    prelude::{ActorStopReason, Context, WeakActorRef},
};
use std::{collections::HashMap, ops::ControlFlow};
use tracing::info;

#[derive(Default)]
pub struct OperatorRegistry {
    connected: HashMap<ActorId, ActorRef<OperatorSession>>,
}

impl Actor for OperatorRegistry {
    type Args = Self;

    type Error = Infallible;

    async fn on_start(args: Self::Args, _: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(args)
    }

    async fn on_link_died(
        &mut self,
        _: WeakActorRef<Self>,
        id: ActorId,
        _: ActorStopReason,
    ) -> Result<ControlFlow<ActorStopReason>, Self::Error> {
        if self.connected.remove(&id).is_some() {
            info!(
                ?id,
                actor = "OperatorRegistry",
                event = "operator.disconnected"
            );
        }
        Ok(ControlFlow::Continue(()))
    }
}

#[messages]
impl OperatorRegistry {
    #[message(ctx)]
    pub async fn connect_operator(
        &mut self,
        actor: ActorRef<OperatorSession>,
        ctx: &mut Context<Self, ()>,
    ) {
        info!(id = %actor.id(), actor = "OperatorRegistry", event = "operator.connected");
        ctx.actor_ref().link(&actor).await;
        self.connected.insert(actor.id(), actor);
    }

    #[message]
    pub fn get_connected(&self) -> Vec<ActorRef<OperatorSession>> {
        self.connected.values().cloned().collect()
    }
}
