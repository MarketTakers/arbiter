use tonic::Status;

use crate::db;

#[derive(Debug, thiserror::Error)]
pub enum UserAgentError {
    #[error("Missing payload in request")]
    MissingPayload,

    #[error("Invalid bootstrap token")]
    InvalidBootstrapToken,

    #[error("Public key not registered")]
    PubkeyNotRegistered,

    #[error("Invalid public key format")]
    InvalidPubkey,

    #[error("Invalid signature length")]
    InvalidSignatureLength,

    #[error("Invalid challenge solution")]
    InvalidChallengeSolution,

    #[error("Invalid state for operation")]
    InvalidState,

    #[error("Actor unavailable")]
    ActorUnavailable,

    #[error("Database error")]
    Database(#[from] diesel::result::Error),

    #[error("Database pool error")]
    DatabasePool(#[from] db::PoolError),
}

impl From<UserAgentError> for Status {
    fn from(err: UserAgentError) -> Self {
        match err {
            UserAgentError::MissingPayload
            | UserAgentError::InvalidBootstrapToken
            | UserAgentError::InvalidPubkey
            | UserAgentError::InvalidSignatureLength => Status::invalid_argument(err.to_string()),

            UserAgentError::PubkeyNotRegistered | UserAgentError::InvalidChallengeSolution => {
                Status::unauthenticated(err.to_string())
            }

            UserAgentError::InvalidState => Status::failed_precondition(err.to_string()),

            UserAgentError::ActorUnavailable
            | UserAgentError::Database(_)
            | UserAgentError::DatabasePool(_) => Status::internal(err.to_string()),
        }
    }
}
