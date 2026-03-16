
use chacha20poly1305::aead::KeyInit;
use ed25519_dalek::VerifyingKey;
use kameo::{Actor, messages, prelude::Context};
use tokio::{select, sync::watch};
use tracing::{error, info};

use crate::actors::{
    router::RegisterUserAgent,
    user_agent::{
        Request, Response, TransportResponseError,
        UserAgentConnection,
    },
};

mod state;
use state::{DummyContext, UserAgentEvents, UserAgentStateMachine};

// Error for consumption by other actors
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("User agent session ended due to connection loss")]
    ConnectionLost,

    #[error("User agent session ended due to unexpected message")]
    UnexpectedMessage,
}

pub struct UserAgentSession {
    props: UserAgentConnection,
    state: UserAgentStateMachine<DummyContext>,
}

mod connection;

impl UserAgentSession {
    pub(crate) fn new(props: UserAgentConnection) -> Self {
        Self {
            props,
            state: UserAgentStateMachine::new(DummyContext),
        }
    }

    pub(super) async fn send_msg<Reply: kameo::Reply>(
        &mut self,
        msg: Response,
        _ctx: &mut Context<Self, Reply>,
    ) -> Result<(), Error> {
        self.props.transport.send(Ok(msg)).await.map_err(|_| {
            error!(
                actor = "useragent",
                reason = "channel closed",
                "send.failed"
            );
            Error::ConnectionLost
        })
    }

    async fn expect_msg<Extractor, Msg, Reply>(
        &mut self,
        extractor: Extractor,
        ctx: &mut Context<Self, Reply>,
    ) -> Result<Msg, Error>
    where
        Extractor: FnOnce(Request) -> Option<Msg>,
        Reply: kameo::Reply,
    {
        let msg = self.props.transport.recv().await.ok_or_else(|| {
            error!(
                actor = "useragent",
                reason = "channel closed",
                "recv.failed"
            );
            ctx.stop();
            Error::ConnectionLost
        })?;

        extractor(msg).ok_or_else(|| {
            error!(
                actor = "useragent",
                reason = "unexpected message",
                "recv.failed"
            );
            ctx.stop();
            Error::UnexpectedMessage
        })
    }

    fn transition(&mut self, event: UserAgentEvents) -> Result<(), TransportResponseError> {
        self.state.process_event(event).map_err(|e| {
            error!(?e, "State transition failed");
            TransportResponseError::StateTransitionFailed
        })?;
        Ok(())
    }
}

#[messages]
impl UserAgentSession {
    // TODO: Think about refactoring it to state-machine based flow, as we already have one
    #[message(ctx)]
    pub async fn request_new_client_approval(
        &mut self,
        client_pubkey: VerifyingKey,
        mut cancel_flag: watch::Receiver<()>,
        ctx: &mut Context<Self, Result<bool, Error>>,
    ) -> Result<bool, Error> {
        self.send_msg(
            Response::ClientConnectionRequest {
                pubkey: client_pubkey,
            },
            ctx,
        )
        .await?;

        let extractor = |msg| {
            if let Request::ClientConnectionResponse { approved } = msg {
                Some(approved)
            } else {
                None
            }
        };

        tokio::select! {
            _ = cancel_flag.changed() => {
                info!(actor = "useragent", "client connection approval cancelled");
                self.send_msg(
                    Response::ClientConnectionCancel,
                    ctx,
                ).await?;
                Ok(false)
            }
            result = self.expect_msg(extractor, ctx) => {
                let result = result?;
                info!(actor = "useragent", "received client connection approval result: approved={}", result);
                Ok(result)
            }
        }
    }
}

impl Actor for UserAgentSession {
    type Args = Self;

    type Error = TransportResponseError;

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
                TransportResponseError::ConnectionRegistrationFailed
            })?;
        Ok(args)
    }

    async fn next(
        &mut self,
        _actor_ref: kameo::prelude::WeakActorRef<Self>,
        mailbox_rx: &mut kameo::prelude::MailboxReceiver<Self>,
    ) -> Option<kameo::mailbox::Signal<Self>> {
        loop {
            select! {
                signal = mailbox_rx.recv() => {
                    return signal;
                }
                msg = self.props.transport.recv() => {
                    match msg {
                        Some(request) => {
                            match self.process_transport_inbound(request).await {
                                Ok(response) => {
                                    if self.props.transport.send(Ok(response)).await.is_err() {
                                        error!(actor = "useragent", reason = "channel closed", "send.failed");
                                        return Some(kameo::mailbox::Signal::Stop);
                                    }
                                }
                                Err(err) => {
                                    let _ = self.props.transport.send(Err(err)).await;
                                    return Some(kameo::mailbox::Signal::Stop);
                                }
                            }
                        }
                        None => {
                            info!(actor = "useragent", "transport.closed");
                            return Some(kameo::mailbox::Signal::Stop);
                        }
                    }
                }
            }
        }
    }
}

impl UserAgentSession {
    pub fn new_test(db: crate::db::DatabasePool, actors: crate::actors::GlobalActors) -> Self {
        use arbiter_proto::transport::DummyTransport;
        let transport: super::Transport = Box::new(DummyTransport::new());
        let props = UserAgentConnection::new(db, actors, transport);
        Self {
            props,
            state: UserAgentStateMachine::new(DummyContext),
        }
    }
}
