use super::{OutOfBand, OperatorConnection};
use crate::{
    actors::{
        flow_coordinator::{GetConnectedClientIds, client_connect_approval::ClientApprovalController},
        operator_registry::ConnectOperator,
    },
    peers::client::ClientProfile,
};
use arbiter_crypto::authn;
use arbiter_proto::transport::Sender;

use kameo::{Actor, actor::ActorRef, messages};
use std::{borrow::Cow, collections::{HashMap, HashSet}};
use thiserror::Error;
use tracing::error;

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

pub struct OperatorSession {
    props: OperatorConnection,
    sender: Box<dyn Sender<OutOfBand>>,

    pending_client_approvals: HashMap<Vec<u8>, PendingClientApproval>,
    /// DB client_ids this operator session is allowed to sign for.
    /// Seeded from currently-connected clients on start, then updated as
    /// approvals are granted or denied during the session lifetime.
    approved_client_ids: HashSet<i32>,
}

pub mod handlers;

impl OperatorSession {
    pub(crate) fn new(props: OperatorConnection, sender: Box<dyn Sender<OutOfBand>>) -> Self {
        Self {
            props,
            sender,
            pending_client_approvals: HashMap::default(),
            approved_client_ids: HashSet::default(),
        }
    }
}

#[messages]
impl OperatorSession {
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
                actor = "operator",
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

impl Actor for OperatorSession {
    type Args = Self;

    type Error = Error;

    async fn on_start(mut args: Self::Args, this: ActorRef<Self>) -> Result<Self, Self::Error> {
        args.props
            .actors
            .operator_registry
            .ask(ConnectOperator {
                actor: this.clone(),
            })
            .await
            .map_err(|err| {
                error!(
                    ?err,
                    "Failed to register operator connection with operator registry"
                );
                Error::internal("Failed to register operator connection with operator registry")
            })?;

        // Seed approved set with clients already connected when this session starts.
        // New clients will be added via handle_new_client_approve as they are approved.
        match args.props.actors.flow_coordinator.ask(GetConnectedClientIds {}).await {
            Ok(ids) => args.approved_client_ids.extend(ids),
            Err(err) => {
                error!(?err, "Failed to fetch connected client IDs on operator session start");
            }
        }

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
                    actor = "operator",
                    event = "failed to announce client connection cancellation"
                );
            }
        }

        Ok(std::ops::ControlFlow::Continue(()))
    }
}
