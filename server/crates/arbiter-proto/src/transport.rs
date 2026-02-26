//! Transport-facing abstractions for protocol/session code.
//!
//! This module separates three concerns:
//!
//! - protocol/session logic wants a small duplex interface ([`Bi`])
//! - transport adapters push concrete stream items to an underlying IO layer
//! - transport boundaries translate between protocol-facing and transport-facing
//!   item types via direction-specific converters
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
//! [`RecvConverter`] and [`SendConverter`] are infallible conversion traits used
//! by adapters to map between protocol-facing and transport-facing item types.
//! The traits themselves are not result-aware; adapters decide how transport
//! errors are handled before (or instead of) conversion.
//!
//! [`grpc::GrpcAdapter`] combines:
//! - a tonic inbound stream
//! - a Tokio sender for outbound transport items
//! - a [`RecvConverter`] for the receive path
//! - a [`SendConverter`] for the send path
//!
//! [`DummyTransport`] is a no-op implementation useful for tests and local actor
//! execution where no real network stream exists.
//!
//! # Component Interaction
//!
//! ```text
//! inbound (network -> protocol)
//! ============================
//!
//! tonic::Streaming<RecvTransport>
//!     -> grpc::GrpcAdapter::recv()
//!          |
//!          +--> on `Ok(item)`: RecvConverter::convert(RecvTransport) -> Inbound
//!          +--> on `Err(status)`: log error and close stream (`None`)
//!     -> Bi::recv()
//!     -> protocol/session actor
//!
//! outbound (protocol -> network)
//! ==============================
//!
//! protocol/session actor
//!     -> Bi::send(Outbound)
//!     -> grpc::GrpcAdapter::send()
//!          |
//!          +--> SendConverter::convert(Outbound) -> SendTransport
//!     -> Tokio mpsc::Sender<SendTransport>
//!     -> tonic response stream
//! ```
//!
//! # Design Notes
//!
//! - `send()` returns [`Error`] only for transport delivery failures (for
//!   example, when the outbound channel is closed).
//! - [`grpc::GrpcAdapter`] logs tonic receive errors and treats them as stream
//!   closure (`None`).
//! - When protocol-facing and transport-facing types are identical, use
//!   [`IdentityRecvConverter`] / [`IdentitySendConverter`].

use std::marker::PhantomData;

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
pub trait Bi<Inbound, Outbound>: Send + Sync + 'static {
    fn send(
        &mut self,
        item: Outbound,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;

    fn recv(&mut self) -> impl std::future::Future<Output = Option<Inbound>> + Send;
}

/// Converts transport-facing inbound items into protocol-facing inbound items.
pub trait RecvConverter: Send + Sync + 'static {
    type Input;
    type Output;

    fn convert(&self, item: Self::Input) -> Self::Output;
}

/// Converts protocol/domain outbound items into transport-facing outbound items.
pub trait SendConverter: Send + Sync + 'static {
    type Input;
    type Output;

    fn convert(&self, item: Self::Input) -> Self::Output;
}

/// A [`RecvConverter`] that forwards values unchanged.
pub struct IdentityRecvConverter<T> {
    _marker: PhantomData<T>,
}

impl<T> IdentityRecvConverter<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for IdentityRecvConverter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> RecvConverter for IdentityRecvConverter<T>
where
    T: Send + Sync + 'static,
{
    type Input = T;
    type Output = T;

    fn convert(&self, item: Self::Input) -> Self::Output {
        item
    }
}

/// A [`SendConverter`] that forwards values unchanged.
pub struct IdentitySendConverter<T> {
    _marker: PhantomData<T>,
}

impl<T> IdentitySendConverter<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for IdentitySendConverter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SendConverter for IdentitySendConverter<T>
where
    T: Send + Sync + 'static,
{
    type Input = T;
    type Output = T;

    fn convert(&self, item: Self::Input) -> Self::Output {
        item
    }
}

/// gRPC-specific transport adapters and helpers.
pub mod grpc {
    use futures::StreamExt;
    use tokio::sync::mpsc;
    use tonic::Streaming;

    use super::{Bi, Error, RecvConverter, SendConverter};

    /// [`Bi`] adapter backed by a tonic gRPC bidirectional stream.
    ///

    /// Tonic receive errors are logged and treated as stream closure (`None`).
    /// The receive converter is only invoked for successful inbound transport
    /// items.
    pub struct GrpcAdapter<InboundConverter, OutboundConverter>
    where
        InboundConverter: RecvConverter,
        OutboundConverter: SendConverter,
    {
        sender: mpsc::Sender<OutboundConverter::Output>,
        receiver: Streaming<InboundConverter::Input>,
        inbound_converter: InboundConverter,
        outbound_converter: OutboundConverter,
    }


    impl<InboundTransport, Inbound, InboundConverter, OutboundConverter>
        GrpcAdapter<InboundConverter, OutboundConverter>
    where
        InboundConverter: RecvConverter<Input = InboundTransport, Output = Inbound>,
        OutboundConverter: SendConverter,
    {
        pub fn new(
            sender: mpsc::Sender<OutboundConverter::Output>,
            receiver: Streaming<InboundTransport>,
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


    impl< InboundConverter, OutboundConverter> Bi<InboundConverter::Output, OutboundConverter::Input>
        for GrpcAdapter<InboundConverter, OutboundConverter>
    where
        InboundConverter: RecvConverter,
        OutboundConverter: SendConverter,
        OutboundConverter::Input: Send + 'static,
        OutboundConverter::Output: Send + 'static,
    {
        #[tracing::instrument(level = "trace", skip(self, item))]
        async fn send(&mut self, item: OutboundConverter::Input) -> Result<(), Error> {
            let outbound = self.outbound_converter.convert(item);
            self.sender
                .send(outbound)
                .await
                .map_err(|_| Error::ChannelClosed)
        }

        #[tracing::instrument(level = "trace", skip(self))]
        async fn recv(&mut self) -> Option<InboundConverter::Output> {
            match self.receiver.next().await {
                Some(Ok(item)) => Some(self.inbound_converter.convert(item)),
                Some(Err(error)) => {
                    tracing::error!(error = ?error, "grpc transport recv failed; closing stream");
                    None
                }
                None => None,
            }
        }
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
