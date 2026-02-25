use arbiter_proto::{proto::UserAgentRequest, transport::TransportActor};
use ed25519_dalek::SigningKey;
use kameo::{
    Actor, Reply,
    actor::{ActorRef, WeakActorRef},
    prelude::Message,
};
use smlang::statemachine;
use tonic::transport::CertificateDer;
use tracing::{debug, error};

struct Storage {
    pub identity: SigningKey,
    pub server_ca_cert: CertificateDer<'static>,
}

#[derive(Debug)]
pub enum InitError {
    StorageError,
    Other(String),
}

statemachine! {
    name: UserAgentStateMachine,
    custom_error: false,
    transitions: {
        *Init + SendAuthChallenge = WaitingForAuthSolution
    }
}



pub struct UserAgentActor<A: TransportActor<UserAgentRequest>> {
    key: SigningKey,
    server_ca_cert: CertificateDer<'static>,
    sender: ActorRef<A>,
}
impl<A: TransportActor<UserAgentRequest>> Actor for UserAgentActor<A> {
    type Args = Self;

    type Error = InitError;

    async fn on_start(args: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        todo!()
    }

    async fn on_link_died(
        &mut self,
        _: WeakActorRef<Self>,
        id: kameo::prelude::ActorId,
        _: kameo::prelude::ActorStopReason,
    ) -> Result<std::ops::ControlFlow<kameo::prelude::ActorStopReason>, Self::Error> {
        if id == self.sender.id() {
            error!("Transport actor died, stopping UserAgentActor");
            Ok(std::ops::ControlFlow::Break(
                kameo::prelude::ActorStopReason::Normal,
            ))
        } else {
            debug!(
                "Linked actor {} died, but it's not the transport actor, ignoring",
                id
            );
            Ok(std::ops::ControlFlow::Continue(()))
        }
    }
}
