//! Transport-facing abstractions for protocol/session code.
//!
//! This module separates three concerns:
//!
//! - protocol/session logic wants a small duplex interface ([`Bi`])
//! - transport adapters need to push concrete stream items to an underlying IO layer
//! - server/client boundaries may need to translate domain outbounds into transport
//!   framing (for example, a tonic stream item)
//!
//! [`Bi`] is intentionally minimal and transport-agnostic:
//! - [`Bi::recv`] yields inbound protocol messages
//! - [`Bi::send`] accepts outbound protocol/domain items
//!
//! # Generic Ordering Rule
//!
//! This module uses a single convention consistently: when a type or trait is
//! parameterized by protocol message directions, the generic parameters are
//! declared as `Inbound` first, then `Outbound`.
//!
//! For [`Bi`], that means `Bi<Inbound, Outbound>`:
//! - `recv() -> Option<Inbound>`
//! - `send(Outbound)`
//!
//! For adapter types that are parameterized by direction-specific converters,
//! inbound-related converter parameters are declared before outbound-related
//! converter parameters.
//!
//! [`ProtocolConverter`] is the boundary object that converts a protocol/domain
//! outbound item into the concrete outbound item expected by a transport sender.
//! The conversion is infallible, so domain-level recoverable failures should be
//! represented inside the domain outbound type itself (for example,
//! `Result<Message, DomainError>`).
//!
//! [`GrpcAdapter`] combines:
//! - a tonic inbound stream
//! - a Tokio sender for outbound transport items
//! - a [`ProtocolConverter`] for the receive path
//! - a [`ProtocolConverter`] for the send path
//!
//! [`DummyTransport`] is a no-op implementation useful for tests and local actor
//! execution where no real network stream exists.
//!
//! # Component Interaction
//!
//! The typical layering looks like this:
//!
//! ```text
//! inbound (network -> protocol)
//! ============================
//!
//!  tonic::Streaming<RecvTransport>  -> GrpcAdapter::recv()  -> Bi::recv()  -> protocol/session actor
//!                                         |
//!                                         +--> recv ProtocolConverter::convert(transport)
//! 
//! outbound (protocol -> network)
//! ==============================
//!
//!  protocol/session actor  -> Bi::send(domain outbound item, e.g. Result<Message, DomainError>)
//!                          -> GrpcAdapter::send()
//!                               |
//!                               +--> send ProtocolConverter::convert(domain)
//!                          -> Tokio mpsc::Sender<SendTransport>  -> tonic response stream
//! ```
//!
//! # Design Notes
//!
//! - `recv()` collapses adapter-specific receive failures into `None`, which
//!   lets protocol code treat stream termination and transport receive failure as
//!   "no more inbound items" when no finer distinction is required.
//! - `send()` returns [`Error`] only for transport delivery failures (for example,
//!   when the outbound channel is closed).
//! - Conversion policy lives outside protocol/session logic and can be defined at
//!   the transport boundary (such as a server endpoint module). When domain and
//!   transport types are identical, [`IdentityConverter`] can be used.

use std::marker::PhantomData;

use futures::StreamExt;
use tokio::sync::mpsc;
use tonic::Streaming;

/// Errors returned by transport adapters implementing [`Bi`].
pub enum Error {
    /// The outbound side of the transport is no longer accepting messages.
    ChannelClosed,
}

/// Minimal bidirectional transport abstraction used by protocol code.
///
/// `Bi<Inbound, Outbound>` models a duplex channel with:
/// - inbound items of type `Inbound` read via [`Bi::recv`]
/// - outbound items of type `Outbound` written via [`Bi::send`]
///
/// The trait intentionally exposes only the operations the protocol layer needs,
/// allowing it to work with gRPC streams and other transport implementations.
///
/// # Stream termination and errors
///
/// [`Bi::recv`] returns:
/// - `Some(item)` when a new inbound message is available
/// - `None` when the inbound stream ends or the underlying transport reports an error
///
/// Implementations may collapse transport-specific receive errors into `None`
/// when the protocol does not need to distinguish them from normal stream
/// termination.
pub trait Bi<Inbound, Outbound>: Send + Sync + 'static {
    /// Sends one outbound item to the peer.
    fn send(
        &mut self,
        item: Outbound,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;

    /// Receives the next inbound item.
    ///
    /// Returns `None` when the inbound stream is finished or can no longer
    /// produce items.
    fn recv(&mut self) -> impl std::future::Future<Output = Option<Inbound>> + Send;
}

/// Converts protocol/domain outbound items into transport-layer outbound items.
///
/// This trait is used by transport adapters that need to emit a concrete stream
/// item type (for example, tonic server streams) while protocol code prefers to
/// work with domain-oriented outbound values.
///
/// `convert` is infallible by design. Any recoverable protocol failure should be
/// represented in [`Self::Domain`] and mapped into the transport item in the
/// converter implementation.
pub trait ProtocolConverter: Send + Sync + 'static {
    /// Outbound item produced by protocol/domain code.
    type Domain;

    /// Outbound item required by the transport sender.
    type Transport;

    /// Maps a protocol/domain outbound item into the transport sender item.
    fn convert(&self, item: Self::Domain) -> Self::Transport;
}

/// A [`ProtocolConverter`] that forwards values unchanged.
///
/// Useful when the protocol-facing and transport-facing item types are
/// identical, but a converter is still required by an adapter API.
pub struct IdentityConverter<T> {
    _marker: PhantomData<T>,
}

impl<T> IdentityConverter<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for IdentityConverter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ProtocolConverter for IdentityConverter<T>
where
    T: Send + Sync + 'static,
{
    type Domain = T;
    type Transport = T;

    fn convert(&self, item: Self::Domain) -> Self::Transport {
        item
    }
}

/// [`Bi`] adapter backed by a tonic gRPC bidirectional stream.
///
/// The adapter owns converter instances for both directions:
/// - receive converter: transport inbound -> protocol inbound
/// - send converter: protocol outbound -> transport outbound
///
/// This keeps protocol actors decoupled from transport framing conventions in
/// both directions.
pub struct GrpcAdapter<InboundConverter: ProtocolConverter, OutboundConverter: ProtocolConverter> {
    sender: mpsc::Sender<OutboundConverter::Transport>,
    receiver: Streaming<InboundConverter::Domain>,
    inbound_converter: InboundConverter,
    outbound_converter: OutboundConverter,
}

impl<InboundConverter, OutboundConverter> GrpcAdapter<InboundConverter, OutboundConverter>
where
    InboundConverter: ProtocolConverter,
    OutboundConverter: ProtocolConverter,
{
    /// Creates a new gRPC-backed [`Bi`] adapter.
    ///
    /// The provided converters define:
    /// - the protocol outbound item and corresponding transport outbound item
    /// - the transport inbound item and corresponding protocol inbound item
    pub fn new(
        sender: mpsc::Sender<OutboundConverter::Transport>,
        receiver: Streaming<InboundConverter::Domain>,
        inbound_converter: InboundConverter,
        outbound_converter: OutboundConverter,
    ) -> Self {
        Self {
            sender,
            receiver,
            inbound_converter,
            outbound_converter,
        }
    }
}

impl<InboundConverter, OutboundConverter>
    Bi<InboundConverter::Transport, OutboundConverter::Domain>
    for GrpcAdapter<InboundConverter, OutboundConverter>
where
    InboundConverter: ProtocolConverter,
    OutboundConverter: ProtocolConverter,
    OutboundConverter::Domain: Send + 'static,
    OutboundConverter::Transport: Send + 'static,
    InboundConverter::Transport: Send + 'static,
    InboundConverter::Domain: Send + 'static,
{
    #[tracing::instrument(level = "trace", skip(self, item))]
    async fn send(&mut self, item: OutboundConverter::Domain) -> Result<(), Error> {
        let outbound: OutboundConverter::Transport = self.outbound_converter.convert(item);
        self.sender
            .send(outbound)
            .await
            .map_err(|_| Error::ChannelClosed)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn recv(&mut self) -> Option<InboundConverter::Transport> {
        self.receiver
            .next()
            .await
            .transpose()
            .ok()
            .flatten()
            .map(|item| self.inbound_converter.convert(item))
    }
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

impl<Inbound, Outbound> Bi<Inbound, Outbound> for DummyTransport<Inbound, Outbound>
where
    Inbound: Send + Sync + 'static,
    Outbound: Send + Sync + 'static,
{
    async fn send(&mut self, _item: Outbound) -> Result<(), Error> {
        Ok(())
    }

    fn recv(&mut self) -> impl std::future::Future<Output = Option<Inbound>> + Send {
        async {
            std::future::pending::<()>().await;
            None
        }
    }
}
