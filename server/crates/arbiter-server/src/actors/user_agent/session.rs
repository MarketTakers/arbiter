use std::{borrow::Cow, convert::Infallible};

use arbiter_proto::transport::Sender;
use ed25519_dalek::VerifyingKey;
use kameo::{Actor, messages, prelude::Context};
use thiserror::Error;
use tokio::{select, sync::watch};
use tracing::{error, info};

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
    sender: Box<dyn Sender<OutOfBand>>,
}

mod connection;
pub(crate) use connection::{
    BootstrapError, HandleBootstrapEncryptedKey, HandleEvmWalletCreate, HandleEvmWalletList,
    HandleGrantCreate, HandleGrantDelete, HandleGrantList, HandleQueryVaultState,
    HandleUnsealEncryptedKey, HandleUnsealRequest, UnsealError,
};

impl UserAgentSession {
    pub(crate) fn new(props: UserAgentConnection, sender: Box<dyn Sender<OutOfBand>>) -> Self {
        Self {
            props,
            state: UserAgentStateMachine::new(DummyContext),
            sender,
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
    #[message(ctx)]
    pub async fn request_new_client_approval(
        &mut self,
        client_pubkey: VerifyingKey,
        mut cancel_flag: watch::Receiver<()>,
        ctx: &mut Context<Self, Result<bool, ()>>,
    ) -> Result<bool, ()> {
        todo!("Think about refactoring it to state-machine based flow, as we already have one")
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
