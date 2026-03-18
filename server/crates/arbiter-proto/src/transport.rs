//! Transport-facing abstractions shared by protocol/session code.
//!
//! This module defines a small duplex interface, [`Bi`], that actors and other
//! protocol code can depend on without knowing anything about the concrete
//! transport underneath.
//!
//! [`Bi`] is intentionally minimal and transport-agnostic:
//! - [`Bi::recv`] yields inbound messages
//! - [`Bi::send`] accepts outbound messages
//!
//! Transport-specific adapters, including protobuf or gRPC bridges, live in the
//! crates that own those boundaries rather than in `arbiter-proto`.
//!
//! # Generic Ordering Rule
//!
//! This module consistently uses `Inbound` first and `Outbound` second in
//! generic parameter lists.
//!
//! For [`Bi`], that means `Bi<Inbound, Outbound>`:
//! - `recv() -> Option<Inbound>`
//! - `send(Outbound)`
//!
//! [`expect_message`] is a small helper for request/response style flows: it
//! reads one inbound message from a transport and extracts a typed value from
//! it, failing if the channel closes or the message shape is not what the
//! caller expected.
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

/// Minimal bidirectional transport abstraction used by protocol code.
///
/// `Bi<Inbound, Outbound>` models a duplex channel with:
/// - inbound items of type `Inbound` read via [`Bi::recv`]
/// - outbound items of type `Outbound` written via [`Bi::send`]
#[async_trait]
pub trait Bi<Inbound, Outbound>: Send + Sync + 'static {
    async fn send(&mut self, item: Outbound) -> Result<(), Error>;

    async fn recv(&mut self) -> Option<Inbound>;
}

/// No-op [`Bi`] transport for tests and manual actor usage.
///
/// `send` drops all items and succeeds. [`Bi::recv`] never resolves and therefore
/// does not busy-wait or spuriously close the stream.
pub struct DummyTransport<Inbound, Outbound> {
    _marker: PhantomData<(Inbound, Outbound)>,
}

impl<Inbound, Outbound> DummyTransport<Inbound, Outbound> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<Inbound, Outbound> Default for DummyTransport<Inbound, Outbound> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<Inbound, Outbound> Bi<Inbound, Outbound> for DummyTransport<Inbound, Outbound>
where
    Inbound: Send + Sync + 'static,
    Outbound: Send + Sync + 'static,
{
    async fn send(&mut self, _item: Outbound) -> Result<(), Error> {
        Ok(())
    }

    async fn recv(&mut self) -> Option<Inbound> {
        std::future::pending::<()>().await;
        None
    }
}
