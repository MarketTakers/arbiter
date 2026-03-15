use std::{collections::HashMap, ops::ControlFlow};

use kameo::{
    Actor,
    actor::{ActorId, ActorRef},
    messages,
    prelude::{ActorStopReason, Context, WeakActorRef},
};
use tracing::info;

use crate::actors::{client::session::ClientSession, user_agent::session::UserAgentSession};

#[derive(Default)]
pub struct MessageRouter {
    pub user_agents: HashMap<ActorId, ActorRef<UserAgentSession>>,
    pub clients: HashMap<ActorId, ActorRef<ClientSession>>,
}

impl Actor for MessageRouter {
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
        if self.user_agents.remove(&id).is_some() {
            info!(
                ?id,
                actor = "MessageRouter",
                event = "useragent.disconnected"
            );
        } else if self.clients.remove(&id).is_some() {
            info!(?id, actor = "MessageRouter", event = "client.disconnected");
        } else {
            info!(
                ?id,
                actor = "MessageRouter",
                event = "unknown.actor.disconnected"
            );
        }
        Ok(ControlFlow::Continue(()))
    }
}

#[messages]
impl MessageRouter {
    #[message(ctx)]
    pub async fn register_user_agent(
        &mut self,
        actor: ActorRef<UserAgentSession>,
        ctx: &mut Context<Self, ()>,
    ) {
        info!(id = %actor.id(), actor = "MessageRouter", event = "useragent.connected");
        ctx.actor_ref().link(&actor).await;
        self.user_agents.insert(actor.id(), actor);
    }

    #[message(ctx)]
    pub async fn register_client(
        &mut self,
        actor: ActorRef<ClientSession>,
        ctx: &mut Context<Self, ()>,
    ) {
        info!(id = %actor.id(), actor = "MessageRouter", event = "client.connected");
        ctx.actor_ref().link(&actor).await;
        self.clients.insert(actor.id(), actor);
    }
}
