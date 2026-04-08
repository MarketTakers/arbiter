use std::{collections::HashMap, ops::ControlFlow};

use kameo::{
    Actor,
    actor::{ActorId, ActorRef, Spawn},
    messages,
    prelude::{ActorStopReason, Context, WeakActorRef},
    reply::DelegatedReply,
};
use tracing::info;

use crate::{
    actors::flow_coordinator::client_connect_approval::ClientApprovalController,
    peers::{
        client::{ClientProfile, session::ClientSession},
        user_agent::UserAgentSession,
    },
};

pub mod client_connect_approval;

#[derive(Default)]
pub struct FlowCoordinator {
    pub user_agents: HashMap<ActorId, ActorRef<UserAgentSession>>,
    pub clients: HashMap<ActorId, ActorRef<ClientSession>>,
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
        if self.user_agents.remove(&id).is_some() {
            info!(
                ?id,
                actor = "FlowCoordinator",
                event = "useragent.disconnected"
            );
        } else if self.clients.remove(&id).is_some() {
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
    #[error("No user agents connected")]
    NoUserAgentsConnected,
}

#[messages]
impl FlowCoordinator {
    #[message(ctx)]
    pub async fn register_user_agent(
        &mut self,
        actor: ActorRef<UserAgentSession>,
        ctx: &mut Context<Self, ()>,
    ) {
        info!(id = %actor.id(), actor = "FlowCoordinator", event = "useragent.connected");
        ctx.actor_ref().link(&actor).await;
        self.user_agents.insert(actor.id(), actor);
    }

    #[message(ctx)]
    pub async fn register_client(
        &mut self,
        actor: ActorRef<ClientSession>,
        ctx: &mut Context<Self, ()>,
    ) {
        info!(id = %actor.id(), actor = "FlowCoordinator", event = "client.connected");
        ctx.actor_ref().link(&actor).await;
        self.clients.insert(actor.id(), actor);
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

        let refs: Vec<_> = self.user_agents.values().cloned().collect();
        if refs.is_empty() {
            reply_sender.send(Err(ApprovalError::NoUserAgentsConnected));
            return reply;
        }

        ClientApprovalController::spawn(client_connect_approval::Args {
            client,
            user_agents: refs,
            reply: reply_sender,
        });

        reply
    }
}
