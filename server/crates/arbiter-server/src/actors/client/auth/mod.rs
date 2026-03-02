use arbiter_proto::proto::client::{
    AuthChallengeRequest, AuthChallengeSolution, ClientRequest,
    client_request::Payload as ClientRequestPayload,
};
use ed25519_dalek::VerifyingKey;
use tracing::error;

use crate::actors::client::{
    ConnectionProps,
    auth::state::{AuthContext, AuthStateMachine},
    session::ClientSession,
};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Unexpected message payload")]
    UnexpectedMessagePayload,
    #[error("Invalid client public key length")]
    InvalidClientPubkeyLength,
    #[error("Invalid client public key encoding")]
    InvalidAuthPubkeyEncoding,
    #[error("Database pool unavailable")]
    DatabasePoolUnavailable,
    #[error("Database operation failed")]
    DatabaseOperationFailed,
    #[error("Public key not registered")]
    PublicKeyNotRegistered,
    #[error("Invalid signature length")]
    InvalidSignatureLength,
    #[error("Invalid challenge solution")]
    InvalidChallengeSolution,
    #[error("Transport error")]
    Transport,
}

mod state;
use state::*;

fn parse_auth_event(payload: ClientRequestPayload) -> Result<AuthEvents, Error> {
    match payload {
        ClientRequestPayload::AuthChallengeRequest(AuthChallengeRequest { pubkey }) => {
            let pubkey_bytes = pubkey.as_array().ok_or(Error::InvalidClientPubkeyLength)?;
            let pubkey = VerifyingKey::from_bytes(pubkey_bytes)
                .map_err(|_| Error::InvalidAuthPubkeyEncoding)?;
            Ok(AuthEvents::AuthRequest(ChallengeRequest {
                pubkey: pubkey.into(),
            }))
        }
        ClientRequestPayload::AuthChallengeSolution(AuthChallengeSolution { signature }) => {
            Ok(AuthEvents::ReceivedSolution(ChallengeSolution {
                solution: signature,
            }))
        }
    }
}

pub async fn authenticate(props: &mut ConnectionProps) -> Result<VerifyingKey, Error> {
    let mut state = AuthStateMachine::new(AuthContext::new(props));

    loop {
        let transport = state.context_mut().conn.transport.as_mut();
        let Some(ClientRequest {
            payload: Some(payload),
        }) = transport.recv().await
        else {
            return Err(Error::Transport);
        };

        let event = parse_auth_event(payload)?;

        match state.process_event(event).await {
            Ok(AuthStates::AuthOk(key)) => return Ok(key.clone()),
            Err(AuthError::ActionFailed(err)) => {
                error!(?err, "State machine action failed");
                return Err(err);
            }
            Err(AuthError::GuardFailed(err)) => {
                error!(?err, "State machine guard failed");
                return Err(err);
            }
            Err(AuthError::InvalidEvent) => {
                error!("Invalid event for current state");
                return Err(Error::InvalidChallengeSolution);
            }
            Err(AuthError::TransitionsFailed) => {
                error!("Invalid state transition");
                return Err(Error::InvalidChallengeSolution);
            }

            _ => (),
        }
    }
}

pub async fn authenticate_and_create(
    mut props: ConnectionProps,
) -> Result<ClientSession, Error> {
    let key = authenticate(&mut props).await?;
    let session = ClientSession::new(props, key);
    Ok(session)
}
