use crate::peers::user_agent::UserAgentSession;

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
pub struct UserAgentRegistry {
    connected: HashMap<ActorId, ActorRef<UserAgentSession>>,
}

impl Actor for UserAgentRegistry {
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
                actor = "UserAgentRegistry",
                event = "useragent.disconnected"
            );
        }
        Ok(ControlFlow::Continue(()))
    }
}

#[messages]
impl UserAgentRegistry {
    #[message(ctx)]
    pub async fn connect_useragent(
        &mut self,
        actor: ActorRef<UserAgentSession>,
        ctx: &mut Context<Self, ()>,
    ) {
        info!(id = %actor.id(), actor = "UserAgentRegistry", event = "useragent.connected");
        ctx.actor_ref().link(&actor).await;
        self.connected.insert(actor.id(), actor);
    }

    #[message]
    pub fn get_connected(&self) -> Vec<ActorRef<UserAgentSession>> {
        self.connected.values().cloned().collect()
    }
}
