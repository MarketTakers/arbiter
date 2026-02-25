use futures::{Stream, StreamExt};
use kameo::{
    Actor,
    actor::{ActorRef, PreparedActor, Spawn, WeakActorRef},
    mailbox::Signal,
    prelude::Message,
};
use tokio::{
    select,
    sync::mpsc::{self, error::SendError},
};
use tonic::{Status, Streaming};
use tracing::{debug, error};

// Abstraction for stream for sans-io capabilities
pub trait Bi<T, U>: Stream<Item = Result<T, Status>> + Send + Sync + 'static {
    type Error;
    fn send(
        &mut self,
        item: Result<U, Status>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}

// Bi-directional stream abstraction for handling gRPC streaming requests and responses
pub struct BiStream<T, U> {
    pub request_stream: Streaming<T>,
    pub response_sender: mpsc::Sender<Result<U, Status>>,
}

impl<T, U> Stream for BiStream<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    type Item = Result<T, Status>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.request_stream.poll_next_unpin(cx)
    }
}

impl<T, U> Bi<T, U> for BiStream<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    type Error = SendError<Result<U, Status>>;

    async fn send(&mut self, item: Result<U, Status>) -> Result<(), Self::Error> {
        self.response_sender.send(item).await
    }
}

pub trait TransportActor<T: Send + 'static>: Actor + Send + Message<T> {}

pub struct GrpcTransportActor<SendMsg, RecvMsg, A>
where
    SendMsg: Send + 'static,
    RecvMsg: Send + 'static,
    A: TransportActor<RecvMsg>,
{
    pub sender: mpsc::Sender<Result<SendMsg, tonic::Status>>,
    pub receiver: tonic::Streaming<RecvMsg>,
    pub business_logic_actor: ActorRef<A>,
}
impl<SendMsg, RecvMsg, A> Actor for GrpcTransportActor<SendMsg, RecvMsg, A>
where
    SendMsg: Send + 'static,
    RecvMsg: Send + 'static,
    A: TransportActor<RecvMsg>,
{
    type Args = Self;

    type Error = ();

    async fn on_start(args: Self::Args, _: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(args)
    }

    fn on_link_died(
        &mut self,
        _: WeakActorRef<Self>,
        id: kameo::prelude::ActorId,
        _: kameo::prelude::ActorStopReason,
    ) -> impl Future<
        Output = Result<std::ops::ControlFlow<kameo::prelude::ActorStopReason>, Self::Error>,
    > + Send {
        async move {
            if id == self.business_logic_actor.id() {
                error!("Business logic actor died, stopping GrpcTransportActor");
                Ok(std::ops::ControlFlow::Break(
                    kameo::prelude::ActorStopReason::Normal,
                ))
            } else {
                debug!(
                    "Linked actor {} died, but it's not the business logic actor, ignoring",
                    id
                );
                Ok(std::ops::ControlFlow::Continue(()))
            }
        }
    }

    async fn next(
        &mut self,
        _: WeakActorRef<Self>,
        mailbox_rx: &mut kameo::prelude::MailboxReceiver<Self>,
    ) -> Option<kameo::mailbox::Signal<Self>> {
        select! {
            msg = mailbox_rx.recv() => {
                msg
            }
            recv_msg = self.receiver.next() => {
                match recv_msg {
                    Some(Ok(msg)) => {
                        match self.business_logic_actor.tell(msg).await {
                            Ok(_) => None,
                            Err(e) => {
                                // TODO: this would probably require better error handling - or resending if backpressure is the issue
                                error!("Failed to send message to business logic actor: {}", e);
                                Some(Signal::Stop)
                            }
                        }
                    }
                    Some(Err(e)) => {
                        error!("Received error from stream: {}, stopping GrpcTransportActor", e);
                        Some(Signal::Stop)
                    }
                    None => {
                        error!("Receiver channel closed, stopping GrpcTransportActor");
                        Some(Signal::Stop)
                    }
                }
            }
        }
    }
}

impl<SendMsg: Send + 'static, RecvMsg: Send + 'static, A: TransportActor<RecvMsg>> Message<SendMsg>
    for GrpcTransportActor<SendMsg, RecvMsg, A>
{
    type Reply = ();

    async fn handle(
        &mut self,
        msg: SendMsg,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let err = self.sender.send(Ok(msg)).await;
        match err {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to send message: {}", e);
                ctx.stop();
            }
        }
    }
}

pub async fn wire<T, RecvMsg, SendMsg, BusinessActor, BusinessCtor, TransportCtor>(
    business_ctor: BusinessCtor,
    transport_ctor: TransportCtor,
) -> (ActorRef<T>, ActorRef<BusinessActor>)
where
    T: TransportActor<RecvMsg>,
    RecvMsg: Send + 'static,
    SendMsg: Send + 'static,
    BusinessActor: Actor + Send + 'static,
    BusinessCtor: FnOnce(ActorRef<T>) -> BusinessActor::Args,
    TransportCtor: FnOnce(ActorRef<BusinessActor>) -> T::Args,
{
    let prepared_business: PreparedActor<BusinessActor> = Spawn::prepare();
    let prepared_transport: PreparedActor<T> = Spawn::prepare();

    let business_ref = prepared_business.actor_ref().clone();
    let transport_ref = prepared_transport.actor_ref().clone();

    transport_ref.link(&business_ref).await;
    business_ref.link(&transport_ref).await;

    let _ = prepared_business.spawn(business_ctor(transport_ref.clone()));
    let _ = prepared_transport.spawn(transport_ctor(business_ref.clone()));

    (transport_ref, business_ref)
}
