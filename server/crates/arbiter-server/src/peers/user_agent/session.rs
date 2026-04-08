use arbiter_crypto::authn;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use kameo_actors::message_bus::Register;

use std::{borrow::Cow, collections::HashMap};

use arbiter_proto::transport::Sender;
use async_trait::async_trait;
use kameo::{Actor, actor::ActorRef, messages, prelude::Message};
use thiserror::Error;
use tracing::error;

use crate::{
    actors::{
        flow_coordinator::{RegisterUserAgent, client_connect_approval::ClientApprovalController},
        vault::events,
    }, crypto::integrity, db::schema::useragent_client, peers::{client::ClientProfile, user_agent::UserAgentCredentials}
};
mod state;
use state::{DummyContext, UserAgentEvents, UserAgentStateMachine};

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
    id: i32,
    pubkey: authn::PublicKey,
    props: UserAgentConnection,
    state: UserAgentStateMachine<DummyContext>,
    sender: Box<dyn Sender<OutOfBand>>,

    pending_client_approvals: HashMap<Vec<u8>, PendingClientApproval>,
}

pub mod connection;

impl UserAgentSession {
    pub(crate) fn new(
        props: UserAgentConnection,
        id: i32,
        pubkey: authn::PublicKey,
        sender: Box<dyn Sender<OutOfBand>>,
    ) -> Self {
        Self {
            id,
            props,
            pubkey,
            state: UserAgentStateMachine::new(DummyContext),
            sender,
            pending_client_approvals: Default::default(),
        }
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

        self.pending_client_approvals.insert(
            client.pubkey.to_bytes(),
            PendingClientApproval {
                pubkey: client.pubkey,
                controller,
            },
        );
    }
}

impl Message<events::VaultBootstrapped> for UserAgentSession {
    type Reply = Result<(), Error>;

    async fn handle(
        &mut self,
        _: events::VaultBootstrapped,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let Ok(mut conn) = self.props.db.get().await else {
            error!("Failed to get database connection for vault bootstrapped event");
            ctx.stop();
            return Err(Error::internal("Failed to get database connection"));
        };


        let result = conn.exclusive_transaction(|conn| {
            Box::pin(async {
                let nonce: i32 = useragent_client::table
                    .filter(useragent_client::id.eq(self.id))
                    .select(useragent_client::nonce)
                    .first::<i32>(conn)
                    .await
                    .map_err(|e| {
                        error!(?e, "Failed to get nonce for useragent bootstrapping");
                        Error::internal("Failed to sign user agent credentials")
                    })?;

                let entity = UserAgentCredentials {
                    pubkey: self.pubkey.clone(),
                    nonce,
                };

                integrity::sign_entity(conn, &self.props.actors.vault, &entity, self.id)
                    .await
                    .map_err(|e| {
                        error!(?e, "Failed to sign user agent credentials during vault bootstrapping");
                        Error::internal("Failed to sign user agent credentials")
                    })?;

                Result::<_, Error>::Ok(())
            })
        }).await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => {
                error!(?err, "Error during vault bootstrapping");
                ctx.stop();
                Err(err)
            },
        }

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
            .events
            .tell(Register(
                this.clone().recipient::<events::VaultBootstrapped>(),
            ))
            .await
            .map_err(|err| {
                error!(
                    ?err,
                    "Failed to register user agent connection with event bus"
                );
                Error::internal("Failed to register user agent connection with event bus")
            })?;

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
