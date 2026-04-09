use std::ops::ControlFlow;

use kameo::{
    Actor, messages,
    prelude::{ActorId, ActorRef, ActorStopReason, Context, WeakActorRef},
    reply::ReplySender,
};

use crate::actors::{
    client::ClientProfile,
    flow_coordinator::ApprovalError,
    user_agent::{UserAgentSession, session::BeginNewClientApproval},
};

pub struct Args {
    pub client: ClientProfile,
    pub user_agents: Vec<ActorRef<UserAgentSession>>,
    pub reply: ReplySender<Result<bool, ApprovalError>>,
}

pub struct ClientApprovalController {
    /// Number of UAs that have not yet responded (approval or denial) or died.
    pending: usize,
    /// Number of approvals received so far.
    approved: usize,
    reply: Option<ReplySender<Result<bool, ApprovalError>>>,
}

impl ClientApprovalController {
    fn send_reply(&mut self, result: Result<bool, ApprovalError>) {
        if let Some(reply) = self.reply.take() {
            reply.send(result);
        }
    }
}

impl Actor for ClientApprovalController {
    type Args = Args;
    type Error = ();

    async fn on_start(
        Args {
            client,
            user_agents,
            reply,
        }: Self::Args,
        actor_ref: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let this = Self {
            pending: user_agents.len(),
            approved: 0,
            reply: Some(reply),
        };

        for user_agent in user_agents {
            actor_ref.link(&user_agent).await;

            let _ = user_agent
                .tell(BeginNewClientApproval {
                    client: client.clone(),
                    controller: actor_ref.clone(),
                })
                .await;
        }

        Ok(this)
    }

    async fn on_link_died(
        &mut self,
        _: WeakActorRef<Self>,
        _: ActorId,
        _: ActorStopReason,
    ) -> Result<ControlFlow<ActorStopReason>, Self::Error> {
        // A linked UA died before responding — counts as a non-approval.
        self.pending = self.pending.saturating_sub(1);
        if self.pending == 0 {
            // At least one UA didn't approve: deny.
            self.send_reply(Ok(false));
            return Ok(ControlFlow::Break(ActorStopReason::Normal));
        }
        Ok(ControlFlow::Continue(()))
    }
}

#[messages]
impl ClientApprovalController {
    #[message(ctx)]
    pub fn client_approval_answer(&mut self, approved: bool, ctx: &mut Context<Self, ()>) {
        if !approved {
            // Denial wins immediately regardless of other pending responses.
            self.send_reply(Ok(false));
            ctx.stop();
            return;
        }

        self.approved += 1;
        self.pending = self.pending.saturating_sub(1);

        if self.pending == 0 {
            // Every connected UA approved.
            self.send_reply(Ok(true));
            ctx.stop();
        }
    }
}
