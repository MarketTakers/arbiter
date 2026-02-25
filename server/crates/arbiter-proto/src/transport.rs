//! Transport abstraction layer for bridging gRPC bidirectional streaming with kameo actors.
//!
//! This module provides a clean separation between the gRPC transport layer and business logic
//! by modeling the connection as two linked kameo actors:
//!
//! - A **transport actor** ([`GrpcTransportActor`]) that owns the gRPC stream and channel,
//!   forwarding inbound messages to the business actor and outbound messages to the client.
//! - A **business logic actor** that receives inbound messages from the transport actor and
//!   sends outbound messages back through the transport actor.
//!
//! The [`wire()`] function sets up bidirectional linking between the two actors, ensuring
//! that if either actor dies, the other is notified and can shut down gracefully.
//!
//! # Terminology
//!
//! - **InboundMessage**: a message received by the transport actor from the channel/socket
//!   and forwarded to the business actor.
//! - **OutboundMessage**: a message produced by the business actor and sent to the transport
//!   actor to be forwarded to the channel/socket.
//!
//! # Architecture
//!
//! ```text
//! gRPC Stream ──InboundMessage──▶ GrpcTransportActor ──tell(InboundMessage)──▶ BusinessActor
//!                                 ▲                                    │
//!                                 └─tell(Result<OutboundMessage, _>)────┘
//!                                 │
//!                          mpsc::Sender ──▶ Client
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! let (tx, rx) = mpsc::channel(1000);
//! let context = server_context.clone();
//!
//! wire(
//!     |transport_ref| MyBusinessActor::new(context, transport_ref),
//!     |business_recipient, business_id| GrpcTransportActor {
//!         sender: tx,
//!         receiver: grpc_stream,
//!         business_logic_actor: business_recipient,
//!         business_logic_actor_id: business_id,
//!     },
//! ).await;
//!
//! Ok(Response::new(ReceiverStream::new(rx)))
//! ```

use futures::{Stream, StreamExt};
use kameo::{
    Actor,
    actor::{ActorRef, PreparedActor, Recipient, Spawn, WeakActorRef},
    mailbox::Signal,
    prelude::Message,
};
use tokio::{
    select,
    sync::mpsc::{self, error::SendError},
};
use tonic::{Status, Streaming};
use tracing::{debug, error};

/// A bidirectional stream abstraction for sans-io testing.
///
/// Combines a [`Stream`] of incoming messages with the ability to [`send`](Bi::send)
/// outgoing responses. This trait allows business logic to be tested without a real
/// gRPC connection by swapping in an in-memory implementation.
///
/// # Type Parameters
/// - `T`: `InboundMessage` received from the channel/socket (e.g., `UserAgentRequest`)
/// - `U`: `OutboundMessage` sent to the channel/socket (e.g., `UserAgentResponse`)
pub trait Bi<T, U>: Stream<Item = Result<T, Status>> + Send + Sync + 'static {
    type Error;
    fn send(
        &mut self,
        item: Result<U, Status>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}

/// Concrete [`Bi`] implementation backed by a tonic gRPC [`Streaming`] and an [`mpsc::Sender`].
///
/// This is the production implementation used in gRPC service handlers. The `request_stream`
/// receives messages from the client, and `response_sender` sends responses back.
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

/// Marker trait for transport actors that can receive outbound messages of type `T`.
///
/// Implement this on your transport actor to indicate it can handle outbound messages
/// produced by the business actor. Requires the actor to implement [`Message<Result<T, E>>`]
/// so business logic can forward responses via [`tell()`](ActorRef::tell).
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Actor)]
/// struct MyTransportActor { /* ... */ }
///
/// impl Message<Result<MyResponse, MyError>> for MyTransportActor {
///     type Reply = ();
///     async fn handle(&mut self, msg: Result<MyResponse, MyError>, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
///         // forward outbound message to channel/socket
///     }
/// }
///
/// impl TransportActor<MyResponse, MyError> for MyTransportActor {}
/// ```
pub trait TransportActor<Outbound: Send + 'static, DomainError: Send + 'static>:
    Actor + Send + Message<Result<Outbound, DomainError>>
{
}

/// A kameo actor that bridges a gRPC bidirectional stream with a business logic actor.
///
/// This actor owns the gRPC [`Streaming`] receiver and an [`mpsc::Sender`] for responses.
/// It multiplexes between its own mailbox (for outbound messages from the business actor)
/// and the gRPC stream (for inbound client messages) using [`tokio::select!`].
///
/// # Message Flow
///
/// - **Inbound**: Messages from the gRPC stream are forwarded to `business_logic_actor`
///   via [`tell()`](Recipient::tell).
/// - **Outbound**: The business actor sends `Result<Outbound, DomainError>` messages to this
///   actor, which forwards them through the `sender` channel to the gRPC response stream.
///
/// # Lifecycle
///
/// - If the business logic actor dies (detected via actor linking), this actor stops,
///   which closes the gRPC stream.
/// - If the gRPC stream closes or errors, this actor stops, which (via linking) notifies
///   the business actor.
/// - Error responses (`Err(DomainError)`) are forwarded to the client and then the actor stops,
///   closing the connection.
///
/// # Type Parameters
/// - `Outbound`: `OutboundMessage` sent to the client (e.g., `UserAgentResponse`)
/// - `Inbound`: `InboundMessage` received from the client (e.g., `UserAgentRequest`)
/// - `E`: The domain error type, must implement `Into<tonic::Status>` for gRPC conversion
pub struct GrpcTransportActor<Outbound, Inbound, DomainError>
where
    Outbound: Send + 'static,
    Inbound: Send + 'static,
    DomainError: Into<tonic::Status> + Send + 'static,
{
    sender: mpsc::Sender<Result<Outbound, tonic::Status>>,
    receiver: tonic::Streaming<Inbound>,
    business_logic_actor: Recipient<Inbound>,
    _error: std::marker::PhantomData<DomainError>,
}

impl<Outbound, Inbound, DomainError> GrpcTransportActor<Outbound, Inbound, DomainError>
where
    Outbound: Send + 'static,
    Inbound: Send + 'static,
    DomainError: Into<tonic::Status> + Send + 'static,
{
    pub fn new(
        sender: mpsc::Sender<Result<Outbound, tonic::Status>>,
        receiver: tonic::Streaming<Inbound>,
        business_logic_actor: Recipient<Inbound>,
    ) -> Self {
        Self {
            sender,
            receiver,
            business_logic_actor,
            _error: std::marker::PhantomData,
        }
    }
}

impl<Outbound, Inbound, E> Actor for GrpcTransportActor<Outbound, Inbound, E>
where
    Outbound: Send + 'static,
    Inbound: Send + 'static,
    E: Into<tonic::Status> + Send + 'static,
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

impl<Outbound, Inbound, E> Message<Result<Outbound, E>> for GrpcTransportActor<Outbound, Inbound, E>
where
    Outbound: Send + 'static,
    Inbound: Send + 'static,
    E: Into<tonic::Status> + Send + 'static,
{
    type Reply = ();

    async fn handle(
        &mut self,
        msg: Result<Outbound, E>,
        ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let is_err = msg.is_err();
        let grpc_msg = msg.map_err(Into::into);
        match self.sender.send(grpc_msg).await {
            Ok(_) => {
                if is_err {
                    ctx.stop();
                }
            }
            Err(e) => {
                error!("Failed to send message: {}", e);
                ctx.stop();
            }
        }
    }
}

impl<Outbound, Inbound, E> TransportActor<Outbound, E> for GrpcTransportActor<Outbound, Inbound, E>
where
    Outbound: Send + 'static,
    Inbound: Send + 'static,
    E: Into<tonic::Status> + Send + 'static,
{
}

/// Wires together a transport actor and a business logic actor with bidirectional linking.
///
/// This function handles the chicken-and-egg problem of two actors that need references
/// to each other at construction time. It uses kameo's [`PreparedActor`] to obtain
/// [`ActorRef`]s before spawning, then links both actors so that if either dies,
/// the other is notified via [`on_link_died`](Actor::on_link_died).
///
/// The business actor receives a type-erased [`Recipient<Result<Outbound, DomainError>>`] instead of an
/// `ActorRef<Transport>`, keeping it decoupled from the concrete transport implementation.
///
/// # Type Parameters
/// - `Transport`: The transport actor type (e.g., [`GrpcTransportActor`])
/// - `Inbound`: `InboundMessage` received by the business actor from the transport
/// - `Outbound`: `OutboundMessage` sent by the business actor back to the transport
/// - `Business`: The business logic actor
/// - `BusinessCtor`: Closure that receives a prepared business actor and transport recipient,
///   spawns the business actor, and returns its [`ActorRef`]
/// - `TransportCtor`: Closure that receives a prepared transport actor, a recipient for
///   inbound messages, and the business actor id, then spawns the transport actor
///
/// # Returns
/// A tuple of `(transport_ref, business_ref)` — actor references for both spawned actors.
pub async fn wire<
    Transport,
    Inbound,
    Outbound,
    DomainError,
    Business,
    BusinessCtor,
    TransportCtor,
>(
    business_ctor: BusinessCtor,
    transport_ctor: TransportCtor,
) -> (ActorRef<Transport>, ActorRef<Business>)
where
    Transport: TransportActor<Outbound, DomainError>,
    Inbound: Send + 'static,
    Outbound: Send + 'static,
    DomainError: Send + 'static,
    Business: Actor + Message<Inbound> + Send + 'static,
    BusinessCtor: FnOnce(PreparedActor<Business>, Recipient<Result<Outbound, DomainError>>),
    TransportCtor:
        FnOnce(PreparedActor<Transport>, Recipient<Inbound>),
{
    let prepared_business: PreparedActor<Business> = Spawn::prepare();
    let prepared_transport: PreparedActor<Transport> = Spawn::prepare();

    let business_ref = prepared_business.actor_ref().clone();
    let transport_ref = prepared_transport.actor_ref().clone();

    transport_ref.link(&business_ref).await;
    business_ref.link(&transport_ref).await;

    let recipient = transport_ref.clone().recipient();
    business_ctor(prepared_business, recipient);
    let business_recipient = business_ref.clone().recipient();
    transport_ctor(prepared_transport, business_recipient);


    (transport_ref, business_ref)
}
