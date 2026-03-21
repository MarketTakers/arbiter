use std::{borrow::Cow, collections::HashMap};

use arbiter_proto::transport::Sender;
use async_trait::async_trait;
use ed25519_dalek::VerifyingKey;
use kameo::{Actor, actor::ActorRef, messages, prelude::Context};
use thiserror::Error;
use tracing::error;

use crate::actors::{
    client::ClientProfile,
    flow_coordinator::{RegisterUserAgent, client_connect_approval::ClientApprovalController},
    user_agent::{OutOfBand, UserAgentConnection},
};

mod state;
use state::{DummyContext, UserAgentEvents, UserAgentStateMachine};

#[derive(Debug, Error)]
pub enum Error {
    #[error("State transition failed")]
    State,

    #[error("Internal error: {message}")]
    Internal { message: Cow<'static, str> },
}

impl Error {
    pub fn internal(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

pub struct PendingClientApproval {
    controller: ActorRef<ClientApprovalController>,
}

pub struct UserAgentSession {
    props: UserAgentConnection,
    state: UserAgentStateMachine<DummyContext>,
    sender: Box<dyn Sender<OutOfBand>>,

    pending_client_approvals: HashMap<VerifyingKey, PendingClientApproval>,
}

mod connection;
pub(crate) use connection::{
    BootstrapError, HandleBootstrapEncryptedKey, HandleEvmWalletCreate, HandleEvmWalletList,
    HandleGrantCreate, HandleGrantDelete, HandleGrantList, HandleNewClientApprove,
    HandleQueryVaultState,
};
pub use connection::{HandleUnsealEncryptedKey, HandleUnsealRequest, UnsealError};

impl UserAgentSession {
    pub(crate) fn new(props: UserAgentConnection, sender: Box<dyn Sender<OutOfBand>>) -> Self {
        Self {
            props,
            state: UserAgentStateMachine::new(DummyContext),
            sender,
            pending_client_approvals: Default::default(),
        }
    }

    pub fn new_test(db: crate::db::DatabasePool, actors: crate::actors::GlobalActors) -> Self {
        struct DummySender;

        #[async_trait]
        impl Sender<OutOfBand> for DummySender {
            async fn send(
                &mut self,
                _item: OutOfBand,
            ) -> Result<(), arbiter_proto::transport::Error> {
                Ok(())
            }
        }

        Self::new(UserAgentConnection::new(db, actors), Box::new(DummySender))
    }

    fn transition(&mut self, event: UserAgentEvents) -> Result<(), Error> {
        self.state.process_event(event).map_err(|e| {
            error!(?e, "State transition failed");
            Error::State
        })?;
        Ok(())
    }
}

#[messages]
impl UserAgentSession {
    #[message]
    pub async fn begin_new_client_approval(
        &mut self,
        client: ClientProfile,
        controller: ActorRef<ClientApprovalController>,
    ) {
        if let Err(e) = self
            .sender
            .send(OutOfBand::ClientConnectionRequest {
                profile: client.clone(),
            })
            .await
        {
            error!(
                ?e,
                actor = "user_agent",
                event = "failed to announce new client connection"
            );
            return;
        }

        self.pending_client_approvals
            .insert(client.pubkey, PendingClientApproval { controller });
    }
}

impl Actor for UserAgentSession {
    type Args = Self;

    type Error = Error;

    async fn on_start(
        args: Self::Args,
        this: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        args.props
            .actors
            .flow_coordinator
            .ask(RegisterUserAgent {
                actor: this.clone(),
            })
            .await
            .map_err(|err| {
                error!(
                    ?err,
                    "Failed to register user agent connection with flow coordinator"
                );
                Error::internal("Failed to register user agent connection with flow coordinator")
            })?;
        Ok(args)
    }

    async fn on_link_died(
        &mut self,
        _: kameo::prelude::WeakActorRef<Self>,
        id: kameo::prelude::ActorId,
        _: kameo::prelude::ActorStopReason,
    ) -> Result<std::ops::ControlFlow<kameo::prelude::ActorStopReason>, Self::Error> {
        let cancelled_pubkey = self
            .pending_client_approvals
            .iter()
            .find_map(|(k, v)| (v.controller.id() == id).then_some(*k));

        if let Some(pubkey) = cancelled_pubkey {
            self.pending_client_approvals.remove(&pubkey);

            if let Err(e) = self
                .sender
                .send(OutOfBand::ClientConnectionCancel { pubkey })
                .await
            {
                error!(
                    ?e,
                    actor = "user_agent",
                    event = "failed to announce client connection cancellation"
                );
            }
        }

        Ok(std::ops::ControlFlow::Continue(()))
    }
}
