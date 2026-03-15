use kameo::Actor;
use tokio::select;
use tracing::{error, info};

use crate::{
    actors::{
        GlobalActors,
        client::{ClientConnection, ClientError, Request, Response},
        router::RegisterClient,
    },
    db,
};

pub struct ClientSession {
    props: ClientConnection,
}

impl ClientSession {
    pub(crate) fn new(props: ClientConnection) -> Self {
        Self { props }
    }

    pub async fn process_transport_inbound(&mut self, req: Request) -> Output {
        let _ = req;
        Err(ClientError::UnexpectedRequestPayload)
    }
}

type Output = Result<Response, ClientError>;

impl Actor for ClientSession {
    type Args = Self;

    type Error = ClientError;

    async fn on_start(
        args: Self::Args,
        this: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        args.props
            .actors
            .router
            .ask(RegisterClient { actor: this })
            .await
            .map_err(|_| ClientError::ConnectionRegistrationFailed)?;
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
                                Ok(resp) => {
                                    if self.props.transport.send(Ok(resp)).await.is_err() {
                                        error!(actor = "client", reason = "channel closed", "send.failed");
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
                            info!(actor = "client", "transport.closed");
                            return Some(kameo::mailbox::Signal::Stop);
                        }
                    }
                }
            }
        }
    }
}

impl ClientSession {
    pub fn new_test(db: db::DatabasePool, actors: GlobalActors) -> Self {
        use arbiter_proto::transport::DummyTransport;
        let transport: super::Transport = Box::new(DummyTransport::new());
        let props = ClientConnection::new(db, transport, actors);
        Self { props }
    }
}
