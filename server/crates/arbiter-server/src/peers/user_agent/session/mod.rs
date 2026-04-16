use arbiter_crypto::authn;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{RunQueryDsl};
use kameo_actors::message_bus::Register;

use std::{borrow::Cow, collections::HashMap};

use arbiter_proto::transport::Sender;
use kameo::{Actor, actor::ActorRef, messages, prelude::Message};
use thiserror::Error;
use tracing::error;

use crate::{
    actors::{
        flow_coordinator::client_connect_approval::ClientApprovalController,
        useragent_registry::ConnectUseragent,
        vault::events,
    }, crypto::integrity, db::schema::useragent_client, peers::{client::ClientProfile, user_agent::{AuthCredentials, Credentials}}
};

use super::{OutOfBand, UserAgentConnection};

#[derive(Debug, Error)]
pub enum Error {
    #[error("State transition failed")]
    State,

    #[error("Internal error: {message}")]
    Internal { message: Cow<'static, str> },
}

impl From<crate::db::PoolError> for Error {
    fn from(err: crate::db::PoolError) -> Self {
        error!(?err, "Database pool error");
        Self::internal("Database pool error")
    }
}
impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        error!(?err, "Database error");
        Self::internal("Database error")
    }
}

impl Error {
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
    creds: Credentials,
    props: UserAgentConnection,
    sender: Box<dyn Sender<OutOfBand>>,

    pending_client_approvals: HashMap<Vec<u8>, PendingClientApproval>,
}

pub mod handlers;

impl UserAgentSession {
    pub(crate) fn new(
        props: UserAgentConnection,
        creds: Credentials,
        sender: Box<dyn Sender<OutOfBand>>,
    ) -> Self {
        Self {
            creds,
            props,
            sender,
            pending_client_approvals: Default::default(),
        }
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

    type Error = Error;

    async fn on_start(
        args: Self::Args,
        this: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        args.props
            .actors
            .useragent_registry
            .ask(ConnectUseragent {
                actor: this.clone(),
            })
            .await
            .map_err(|err| {
                error!(
                    ?err,
                    "Failed to register user agent connection with user agent registry"
                );
                Error::internal("Failed to register user agent connection with user agent registry")
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
