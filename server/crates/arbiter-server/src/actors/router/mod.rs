use std::{collections::HashMap, ops::ControlFlow};

use ed25519_dalek::VerifyingKey;
use kameo::{
    Actor,
    actor::{ActorId, ActorRef},
    messages,
    prelude::{ActorStopReason, Context, WeakActorRef},
    reply::DelegatedReply,
};
use tokio::{sync::watch, task::JoinSet};
use tracing::{info, warn};

use crate::actors::{
    client::session::ClientSession,
    user_agent::session::{RequestNewClientApproval, UserAgentSession},
};

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

#[derive(Debug, thiserror::Error)]
pub enum ApprovalError {
    #[error("No user agents connected")]
    NoUserAgentsConnected,
}

async fn request_client_approval(
    user_agents: &[WeakActorRef<UserAgentSession>],
    client_pubkey: VerifyingKey,
) -> Result<bool, ApprovalError> {
    if user_agents.is_empty() {
        return Err(ApprovalError::NoUserAgentsConnected).into();
    }

    let mut pool = JoinSet::new();
    let (cancel_tx, cancel_rx) = watch::channel(());

    for weak_ref in user_agents {
        match weak_ref.upgrade() {
            Some(agent) => {
                let client_pubkey = client_pubkey.clone();
                let cancel_rx = cancel_rx.clone();
                pool.spawn(async move {
                    agent
                        .ask(RequestNewClientApproval {
                            client_pubkey,
                            cancel_flag: cancel_rx.clone(),
                        })
                        .await
                });
            }
            None => {
                warn!(
                    id = weak_ref.id().to_string(),
                    actor = "MessageRouter",
                    event = "useragent.disconnected_before_approval"
                );
            }
        }
    }

    while let Some(result) = pool.join_next().await {
        match result {
            Ok(Ok(approved)) => {
                // cancel other pending requests
                let _ = cancel_tx.send(());
                return Ok(approved);
            }
            Ok(Err(err)) => {
                warn!(
                    ?err,
                    actor = "MessageRouter",
                    event = "useragent.approval_error"
                );
            }
            Err(err) => {
                warn!(
                    ?err,
                    actor = "MessageRouter",
                    event = "useragent.approval_task_failed"
                );
            }
        }
    }

    Err(ApprovalError::NoUserAgentsConnected)
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

    #[message(ctx)]
    pub async fn request_client_approval(
        &mut self,
        client_pubkey: VerifyingKey,
        ctx: &mut Context<Self, DelegatedReply<Result<bool, ApprovalError>>>,
    ) -> DelegatedReply<Result<bool, ApprovalError>> {
        let (reply, Some(reply_sender)) = ctx.reply_sender() else {
            panic!("Exptected `request_client_approval` to have callback channel");
        };

        let weak_refs = self
            .user_agents
            .values()
            .map(|agent| agent.downgrade())
            .collect::<Vec<_>>();

        // handle in subtask to not to lock the actor
        tokio::task::spawn(async move {
            let result = request_client_approval(&weak_refs, client_pubkey).await;
            let _ = reply_sender.send(result);
        });

        reply
    }
}
