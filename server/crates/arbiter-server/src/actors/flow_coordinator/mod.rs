use crate::{
    actors::{
        flow_coordinator::client_connect_approval::ClientApprovalController,
        useragent_registry::{GetConnected, UserAgentRegistry},
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
    useragent_registry: ActorRef<UserAgentRegistry>,
}

impl FlowCoordinator {
    pub fn new(useragent_registry: ActorRef<UserAgentRegistry>) -> Self {
        Self {
            clients: HashMap::default(),
            useragent_registry,
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

        let Ok(refs) = self.useragent_registry.ask(GetConnected).await else {
            reply_sender.send(Err(ApprovalError::NoUserAgentsConnected));
            return reply;
        };

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
