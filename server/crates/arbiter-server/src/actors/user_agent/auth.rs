use arbiter_proto::proto::user_agent::{
    AuthChallengeRequest, AuthChallengeSolution, UserAgentRequest,
    user_agent_request::Payload as UserAgentRequestPayload,
};
use ed25519_dalek::VerifyingKey;
use tracing::error;

use crate::actors::user_agent::{
    ConnectionProps,
    auth::state::{AuthContext, AuthStateMachine}, session::UserAgentSession,
};

#[derive(thiserror::Error, Debug, PartialEq)]
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
    #[error("Transport error")]
    Transport,
    #[error("Invalid bootstrap token")]
    InvalidBootstrapToken,
    #[error("Bootstrapper actor unreachable")]
    BootstrapperActorUnreachable,
    #[error("Invalid challenge solution")]
    InvalidChallengeSolution,
}

mod state;
use state::*;

fn parse_auth_event(payload: UserAgentRequestPayload) -> Result<AuthEvents, Error> {
    match payload {
        UserAgentRequestPayload::AuthChallengeRequest(AuthChallengeRequest {
            pubkey,
            bootstrap_token: None,
        }) => {
            let pubkey_bytes = pubkey.as_array().ok_or(Error::InvalidClientPubkeyLength)?;
            let pubkey = VerifyingKey::from_bytes(pubkey_bytes)
                .map_err(|_| Error::InvalidAuthPubkeyEncoding)?;
            Ok(AuthEvents::AuthRequest(ChallengeRequest {
                pubkey: pubkey.into(),
            }))
        }
        UserAgentRequestPayload::AuthChallengeRequest(AuthChallengeRequest {
            pubkey,
            bootstrap_token: Some(token),
        }) => {
            let pubkey_bytes = pubkey.as_array().ok_or(Error::InvalidClientPubkeyLength)?;
            let pubkey = VerifyingKey::from_bytes(pubkey_bytes)
                .map_err(|_| Error::InvalidAuthPubkeyEncoding)?;
            Ok(AuthEvents::BootstrapAuthRequest(BootstrapAuthRequest {
                pubkey: pubkey.into(),
                token,
            }))
        }
        UserAgentRequestPayload::AuthChallengeSolution(AuthChallengeSolution { signature }) => {
            Ok(AuthEvents::ReceivedSolution(ChallengeSolution {
                solution: signature,
            }))
        }
        _ => Err(Error::UnexpectedMessagePayload),
    }
}

pub async fn authenticate(props: &mut ConnectionProps) -> Result<VerifyingKey, Error> {
    let mut state = AuthStateMachine::new(AuthContext::new(props));

    loop {
        // This is needed because `state` now holds mutable reference to `ConnectionProps`, so we can't directly access `props` here
        let transport = state.context_mut().conn.transport.as_mut();
        let Some(UserAgentRequest {
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


pub async fn authenticate_and_create(mut props: ConnectionProps) -> Result<UserAgentSession, Error> {
    let key = authenticate(&mut props).await?;
    let session = UserAgentSession::new(props, key.clone());
    Ok(session)
}
