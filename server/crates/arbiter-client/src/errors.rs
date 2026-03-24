use terrors::OneOf;
use thiserror::Error;

#[cfg(feature = "evm")]
use alloy::{primitives::ChainId, signers::Error as AlloySignerError};

pub type StorageError = OneOf<(std::io::Error, InvalidKeyLengthError)>;

pub type ConnectError = OneOf<(
    tonic::transport::Error,
    http::uri::InvalidUri,
    webpki::Error,
    tonic::Status,
    MissingAuthChallengeError,
    ApprovalDeniedError,
    NoUserAgentsOnlineError,
    UnexpectedAuthResponseError,
    std::io::Error,
    InvalidKeyLengthError,
)>;

pub type ClientError = OneOf<(tonic::Status, ClientConnectionClosedError)>;

pub(crate) type ClientTransportError =
    OneOf<(TransportChannelClosedError, TransportConnectionClosedError)>;

#[cfg(feature = "evm")]
pub(crate) type EvmWalletError = OneOf<(
    EvmChainIdMismatchError,
    EvmHashSigningUnsupportedError,
    EvmTransactionSigningUnsupportedError,
)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Invalid signing key length in storage: expected {expected} bytes, got {actual} bytes")]
pub struct InvalidKeyLengthError {
    pub expected: usize,
    pub actual: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Auth challenge was not returned by server")]
pub struct MissingAuthChallengeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Client approval denied by User Agent")]
pub struct ApprovalDeniedError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("No User Agents online to approve client")]
pub struct NoUserAgentsOnlineError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Unexpected auth response payload")]
pub struct UnexpectedAuthResponseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Connection closed by server")]
pub struct ClientConnectionClosedError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Transport channel closed")]
pub struct TransportChannelClosedError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Connection closed by server")]
pub struct TransportConnectionClosedError;

#[cfg(feature = "evm")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Transaction chain id mismatch: signer {signer}, tx {tx}")]
pub struct EvmChainIdMismatchError {
    pub signer: ChainId,
    pub tx: ChainId,
}

#[cfg(feature = "evm")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("hash-only signing is not supported for ArbiterEvmWallet; use transaction signing")]
pub struct EvmHashSigningUnsupportedError;

#[cfg(feature = "evm")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("transaction signing is not supported by current arbiter.client protocol")]
pub struct EvmTransactionSigningUnsupportedError;

pub(crate) fn map_auth_code_error(code: i32) -> ConnectError {
    use arbiter_proto::proto::client::AuthResult;

    match AuthResult::try_from(code).unwrap_or(AuthResult::Unspecified) {
        AuthResult::ApprovalDenied => OneOf::new(ApprovalDeniedError),
        AuthResult::NoUserAgentsOnline => OneOf::new(NoUserAgentsOnlineError),
        AuthResult::Unspecified
        | AuthResult::Success
        | AuthResult::InvalidKey
        | AuthResult::InvalidSignature
        | AuthResult::Internal => OneOf::new(UnexpectedAuthResponseError),
    }
}

#[cfg(feature = "evm")]
impl From<EvmChainIdMismatchError> for AlloySignerError {
    fn from(value: EvmChainIdMismatchError) -> Self {
        AlloySignerError::TransactionChainIdMismatch {
            signer: value.signer,
            tx: value.tx,
        }
    }
}

#[cfg(feature = "evm")]
impl From<EvmHashSigningUnsupportedError> for AlloySignerError {
    fn from(_value: EvmHashSigningUnsupportedError) -> Self {
        AlloySignerError::other(
            "hash-only signing is not supported for ArbiterEvmWallet; use transaction signing",
        )
    }
}

#[cfg(feature = "evm")]
impl From<EvmTransactionSigningUnsupportedError> for AlloySignerError {
    fn from(_value: EvmTransactionSigningUnsupportedError) -> Self {
        AlloySignerError::other(
            "transaction signing is not supported by current arbiter.client protocol",
        )
    }
}
