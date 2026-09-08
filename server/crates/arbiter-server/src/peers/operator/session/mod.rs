use super::{AuthenticatedOperator, OutOfBand, OperatorConnection};
use crate::{
    actors::{
        flow_coordinator::client_connect_approval::ClientApprovalController,
        operator_registry::ConnectOperator,
    },
    peers::client::ClientProfile,
};
use arbiter_crypto::authn;
use arbiter_proto::transport::Sender;

use kameo::{Actor, actor::ActorRef, messages};
use std::{borrow::Cow, collections::HashMap};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("State transition failed")]
    State,

    /// §3.5: the ordinary and recovery roles reach for different handlers here. A refusal is a
    /// policy answer, so it is named rather than folded into `Internal` beside real faults.
    #[error("This operator role may not perform that action")]
    RoleNotPermitted,

    /// Fewer access rows were revoked than the request named. Like `RoleNotPermitted` this is
    /// an answer about the request, not a fault, so it is named rather than folded into
    /// `Internal`.
    #[error("Revoked {revoked} of {requested} wallet access entries")]
    PartialRevoke { requested: usize, revoked: usize },

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
    credentials: AuthenticatedOperator,
    sender: Box<dyn Sender<OutOfBand>>,

    pending_client_approvals: HashMap<Vec<u8>, PendingClientApproval>,
}

pub mod handlers;

impl OperatorSession {
    pub(crate) fn new(props: OperatorConnection, credentials: AuthenticatedOperator, sender: Box<dyn Sender<OutOfBand>>) -> Self {
        Self {
            props,
            credentials,
            sender,
            pending_client_approvals: HashMap::default(),
        }
    }

    /// The id of the ordinary operator on the other end, or a refusal.
    ///
    /// Read from the handshake, never from a request body, so a peer cannot act under an id it
    /// did not authenticate as.
    const fn ordinary_id(&self) -> Result<i32, Error> {
        match &self.credentials {
            AuthenticatedOperator::Ordinary(credentials) => Ok(credentials.id),
            AuthenticatedOperator::Recovery(_) => Err(Error::RoleNotPermitted),
        }
    }

    /// The id of the recovery operator on the other end, or a refusal. See `ordinary_id`.
    const fn recovery_id(&self) -> Result<i32, Error> {
        match &self.credentials {
            AuthenticatedOperator::Recovery(credentials) => Ok(credentials.id),
            AuthenticatedOperator::Ordinary(_) => Err(Error::RoleNotPermitted),
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

    async fn on_start(args: Self::Args, this: ActorRef<Self>) -> Result<Self, Self::Error> {
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
