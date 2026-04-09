use arbiter_crypto::authn;

use std::{borrow::Cow, collections::HashMap};

use arbiter_proto::transport::Sender;
use async_trait::async_trait;
use kameo::{Actor, actor::ActorRef, messages};
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
pub enum UserAgentSessionError {
    #[error("Internal error: {message}")]
    Internal { message: Cow<'static, str> },

    #[error("State transition failed")]
    State,
}

impl From<crate::db::PoolError> for UserAgentSessionError {
    fn from(err: crate::db::PoolError) -> Self {
        error!(?err, "Database pool error");
        Self::internal("Database pool error")
    }
}
impl From<diesel::result::Error> for UserAgentSessionError {
    fn from(err: diesel::result::Error) -> Self {
        error!(?err, "Database error");
        Self::internal("Database error")
    }
}

impl UserAgentSessionError {
    pub fn internal(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

pub struct PendingClientApproval {
    pubkey: authn::PublicKey,
    controller: ActorRef<ClientApprovalController>,
}

pub struct UserAgentSession {
    props: UserAgentConnection,
    state: UserAgentStateMachine<DummyContext>,
    sender: Box<dyn Sender<OutOfBand>>,

    pending_client_approvals: HashMap<Vec<u8>, PendingClientApproval>,
}

pub mod connection;

impl UserAgentSession {
    pub(crate) fn new(props: UserAgentConnection, sender: Box<dyn Sender<OutOfBand>>) -> Self {
        Self {
            props,
            state: UserAgentStateMachine::new(DummyContext),
            sender,
            pending_client_approvals: HashMap::default(),
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

    fn transition(&mut self, event: UserAgentEvents) -> Result<(), UserAgentSessionError> {
        self.state.process_event(event).map_err(|e| {
            error!(?e, "State transition failed");
            UserAgentSessionError::State
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

        self.pending_client_approvals.insert(
            client.pubkey.to_bytes(),
            PendingClientApproval {
                pubkey: client.pubkey,
                controller,
            },
        );
    }
}

impl Actor for UserAgentSession {
    type Args = Self;

    type Error = UserAgentSessionError;

    async fn on_start(
        args: Self::Args,
        this: ActorRef<Self>,
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
                UserAgentSessionError::internal(
                    "Failed to register user agent connection with flow coordinator",
                )
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
            .find_map(|(k, v)| (v.controller.id() == id).then_some(k.clone()));

        if let Some(pubkey_bytes) = cancelled_pubkey {
            let Some(approval) = self.pending_client_approvals.remove(&pubkey_bytes) else {
                return Ok(std::ops::ControlFlow::Continue(()));
            };

            if let Err(e) = self
                .sender
                .send(OutOfBand::ClientConnectionCancel {
                    pubkey: approval.pubkey,
                })
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
