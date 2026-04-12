//! Transport-facing abstractions shared by protocol/session code.
//!
//! This module defines a small set of transport traits that actors and other
//! protocol code can depend on without knowing anything about the concrete
//! transport underneath.
//!
//! The abstraction is split into:
//! - [`Sender`] for outbound delivery
//! - [`Receiver`] for inbound delivery
//! - [`Bi`] as the combined duplex form (`Sender + Receiver`)
//!
//! This split lets code depend only on the half it actually needs. For
//! example, some actor/session code only sends out-of-band messages, while
//! auth/state-machine code may need full duplex access.
//!
//! [`Bi`] remains intentionally minimal and transport-agnostic:
//! - [`Receiver::recv`] yields inbound messages
//! - [`Sender::send`] accepts outbound messages
//!
//! Transport-specific adapters, including protobuf or gRPC bridges, live in the
//! crates that own those boundaries rather than in `arbiter-proto`.
//!
//! [`Bi`] deliberately does not model request/response correlation. Some
//! transports may carry multiplexed request/response traffic, some may emit
//! out-of-band messages, and some may be one-message-at-a-time state machines.
//! Correlation concerns such as request IDs, pending response maps, and
//! out-of-band routing belong in the adapter or connection layer built on top
//! of [`Bi`], not in this abstraction itself.
//!
//! # Generic Ordering Rule
//!
//! This module consistently uses `Inbound` first and `Outbound` second in
//! generic parameter lists.
//!
//! For [`Receiver`], [`Sender`], and [`Bi`], this means:
//! - `Receiver<Inbound>`
//! - `Sender<Outbound>`
//! - `Bi<Inbound, Outbound>`
//!
//! Concretely, for [`Bi`]:
//! - `recv() -> Option<Inbound>`
//! - `send(Outbound)`
//!
//! [`expect_message`] is a small helper for linear protocol steps: it reads one
//! inbound message from a transport and extracts a typed value from it, failing
//! if the channel closes or the message shape is not what the caller expected.
//!
//! [`DummyTransport`] is a no-op implementation useful for tests and local
//! actor execution where no real stream exists.
//!
//! # Design Notes
//!
//! - [`Bi::send`] returns [`Error`] only for transport delivery failures, such
//!   as a closed outbound channel.
//! - [`Bi::recv`] returns `None` when the underlying transport closes.
//! - Message translation is intentionally out of scope for this module.

use std::marker::PhantomData;

use async_trait::async_trait;
use kameo::{error::Infallible, prelude::*};

/// Errors returned by transport adapters implementing [`Bi`].
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Transport channel is closed")]
    ChannelClosed,
    #[error("Unexpected message received")]
    UnexpectedMessage,
}

/// Receives one message from `transport` and extracts a value from it using
/// `extractor`. Returns [`Error::ChannelClosed`] if the transport closes and
/// [`Error::UnexpectedMessage`] if `extractor` returns `None`.
pub async fn expect_message<T, Inbound, Outbound, Target, F>(
    transport: &mut T,
    extractor: F,
) -> Result<Target, Error>
where
    T: Bi<Inbound, Outbound> + ?Sized,
    F: FnOnce(Inbound) -> Option<Target>,
{
    let msg = transport.recv().await.ok_or(Error::ChannelClosed)?;
    extractor(msg).ok_or(Error::UnexpectedMessage)
}

#[async_trait]
pub trait Sender<Outbound>: Send + Sync {
    async fn send(&mut self, item: Outbound) -> Result<(), Error>;
}

#[async_trait]
pub trait Receiver<Inbound>: Send + Sync {
    async fn recv(&mut self) -> Option<Inbound>;
}

/// Minimal bidirectional transport abstraction used by protocol code.
///
/// `Bi<Inbound, Outbound>` is the combined duplex form of [`Sender`] and
/// [`Receiver`].
///
/// It models a channel with:
/// - inbound items of type `Inbound` read via [`Bi::recv`]
/// - outbound items of type `Outbound` written via [`Bi::send`]
///
/// It does not imply request/response sequencing, one-at-a-time exchange, or
/// any built-in correlation mechanism between inbound and outbound items.
pub trait Bi<Inbound, Outbound>: Sender<Outbound> + Receiver<Inbound> + Send + Sync {}

#[async_trait]
impl<T, Outbound> Sender<Outbound> for &mut T
where
    T: Sender<Outbound> + ?Sized,
    Outbound: Send + 'static,
{
    async fn send(&mut self, item: Outbound) -> Result<(), Error> {
        (**self).send(item).await
    }
}

#[async_trait]
impl<T, Inbound> Receiver<Inbound> for &mut T
where
    T: Receiver<Inbound> + ?Sized,
    Inbound: Send + 'static,
{
    async fn recv(&mut self) -> Option<Inbound> {
        (**self).recv().await
    }
}

impl<T, Inbound, Outbound> Bi<Inbound, Outbound> for &mut T
where
    T: Bi<Inbound, Outbound> + ?Sized,
    Inbound: Send + 'static,
    Outbound: Send + 'static,
{
}

pub trait SplittableBi<Inbound, Outbound>: Bi<Inbound, Outbound> {
    type Sender: Sender<Outbound>;
    type Receiver: Receiver<Inbound>;

    fn split(self) -> (Self::Sender, Self::Receiver);
    fn from_parts(sender: Self::Sender, receiver: Self::Receiver) -> Self;
}

/// No-op [`Bi`] transport for tests and manual actor usage.
///
/// `send` drops all items and succeeds. [`Bi::recv`] never resolves and therefore
/// does not busy-wait or spuriously close the stream.
pub struct DummyTransport<Inbound, Outbound> {
    _marker: PhantomData<(Inbound, Outbound)>,
}

impl<Inbound, Outbound> Default for DummyTransport<Inbound, Outbound> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<Inbound, Outbound> Sender<Outbound> for DummyTransport<Inbound, Outbound>
where
    Inbound: Send + Sync + 'static,
    Outbound: Send + Sync + 'static,
{
    async fn send(&mut self, _item: Outbound) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl<Inbound, Outbound> Receiver<Inbound> for DummyTransport<Inbound, Outbound>
where
    Inbound: Send + Sync + 'static,
    Outbound: Send + Sync + 'static,
{
    async fn recv(&mut self) -> Option<Inbound> {
        std::future::pending::<()>().await;
        None
    }
}

impl<Inbound, Outbound> Bi<Inbound, Outbound> for DummyTransport<Inbound, Outbound>
where
    Inbound: Send + Sync + 'static,
    Outbound: Send + Sync + 'static,
{
}

pub mod grpc;

#[derive(thiserror::Error, Debug)]
pub enum ForwardError<I> {
    #[error("Transport error: {0}")]
    Transport(#[from] Error),
    #[error("Actor delivery error: {0}")]
    Actor(SendError<I>),
}

pub async fn forward_to_actor<Transport, Inbound, Outbound, Handler>(
    transport: &mut Transport,
    actor: &ActorRef<Handler>,
) -> Result<(), ForwardError<Inbound>>
where
    Transport: Bi<Inbound, <Outbound as Reply>::Ok>,
    Handler: Actor + Message<Inbound, Reply = Outbound>,
    Inbound: Send + 'static,
    Outbound: Send + 'static + Reply<Error = Infallible>, // `Infallible` to enforce contract that `Outbound` carries handler-level error
{
    while let Some(request) = transport.recv().await {
        let resp = actor.ask(request).await.map_err(ForwardError::Actor)?;
        transport.send(resp).await?
    }

    Err(Error::ChannelClosed.into())
}
