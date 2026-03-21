use std::borrow::Cow;

use arbiter_proto::transport::Sender;
use async_trait::async_trait;
use ed25519_dalek::VerifyingKey;
use kameo::{Actor, messages};
use thiserror::Error;
use tokio::sync::watch;
use tracing::error;

use crate::actors::{
    router::RegisterUserAgent,
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

pub struct UserAgentSession {
    props: UserAgentConnection,
    state: UserAgentStateMachine<DummyContext>,
    #[allow(dead_code, reason = "The session keeps ownership of the outbound transport even before the state-machine flow starts using it directly")]
    sender: Box<dyn Sender<OutOfBand>>,
}

mod connection;
pub(crate) use connection::{
    BootstrapError, HandleBootstrapEncryptedKey, HandleEvmWalletCreate, HandleEvmWalletList,
    HandleGrantCreate, HandleGrantDelete, HandleGrantList, HandleQueryVaultState,
};
pub use connection::{HandleUnsealEncryptedKey, HandleUnsealRequest, UnsealError};

impl UserAgentSession {
    pub(crate) fn new(props: UserAgentConnection, sender: Box<dyn Sender<OutOfBand>>) -> Self {
        Self {
            props,
            state: UserAgentStateMachine::new(DummyContext),
            sender,
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
    pub async fn request_new_client_approval(
        &mut self,
        client_pubkey: VerifyingKey,
        mut cancel_flag: watch::Receiver<()>,
    ) -> Result<bool, ()> {
        if self
            .sender
            .send(OutOfBand::ClientConnectionRequest {
                pubkey: client_pubkey,
            })
            .await
            .is_err()
        {
            return Err(());
        }

        let _ = cancel_flag.changed().await;

        let _ = self.sender.send(OutOfBand::ClientConnectionCancel).await;
        Ok(false)
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
            .router
            .ask(RegisterUserAgent {
                actor: this.clone(),
            })
            .await
            .map_err(|err| {
                error!(?err, "Failed to register user agent connection with router");
                Error::internal("Failed to register user agent connection with router")
            })?;
        Ok(args)
    }
}
