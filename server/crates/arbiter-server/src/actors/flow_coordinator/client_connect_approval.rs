use crate::{
    actors::flow_coordinator::ApprovalError,
    peers::{
        client::ClientProfile,
        operator::{OperatorSession, session::BeginNewClientApproval},
    },
};

use kameo::{
    Actor, messages,
    prelude::{ActorId, ActorRef, ActorStopReason, Context, WeakActorRef},
    reply::ReplySender,
};
use std::{ops::ControlFlow, time::Duration};

const APPROVAL_TIMEOUT: Duration = Duration::from_secs(30);

pub struct Args {
    pub client: ClientProfile,
    pub operators: Vec<ActorRef<OperatorSession>>,
    pub reply: ReplySender<Result<bool, ApprovalError>>,
}

pub struct ClientApprovalController {
    /// Number of operators that have not yet responded (approval or denial) or died.
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
            operators,
            reply,
        }: Self::Args,
        actor_ref: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let this = Self {
            pending: operators.len(),
            approved: 0,
            reply: Some(reply),
        };

        for operator in operators {
            actor_ref.link(&operator).await;

            let _ = operator
                .tell(BeginNewClientApproval {
                    client: client.clone(),
                    controller: actor_ref.clone(),
                })
                .await;
        }

        let weak = actor_ref.downgrade();
        tokio::spawn(async move {
            tokio::time::sleep(APPROVAL_TIMEOUT).await;
            if let Some(r) = weak.upgrade() {
                let _ = r.tell(OnApprovalTimeout {}).await;
            }
        });

        Ok(this)
    }

    async fn on_link_died(
        &mut self,
        _: WeakActorRef<Self>,
        _: ActorId,
        _: ActorStopReason,
    ) -> Result<ControlFlow<ActorStopReason>, Self::Error> {
        // A linked operator died before responding — counts as a non-approval.
        self.pending = self.pending.saturating_sub(1);
        if self.pending == 0 {
            // At least one operator didn't approve: deny.
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
            // Every connected operator approved.
            self.send_reply(Ok(true));
            ctx.stop();
        }
    }

    /// Fired after `APPROVAL_TIMEOUT` elapses. Any operator that hasn't responded
    /// by then is treated as a denial to prevent zombie sessions from blocking the flow.
    #[message(ctx)]
    pub fn on_approval_timeout(&mut self, ctx: &mut Context<Self, ()>) {
        if self.pending > 0 {
            self.send_reply(Ok(false));
            ctx.stop();
        }
    }
}
