use tracing::error;

use crate::actors::user_agent::{
    Request, UserAgentConnection,
    auth::state::{AuthContext, AuthStateMachine},
    AuthPublicKey,
    session::UserAgentSession,
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

fn parse_auth_event(payload: Request) -> Result<AuthEvents, Error> {
    match payload {
        Request::AuthChallengeRequest {
            pubkey,
            bootstrap_token: None,
        } => Ok(AuthEvents::AuthRequest(ChallengeRequest { pubkey })),
        Request::AuthChallengeRequest {
            pubkey,
            bootstrap_token: Some(token),
        } => Ok(AuthEvents::BootstrapAuthRequest(BootstrapAuthRequest {
            pubkey,
            token,
        })),
        Request::AuthChallengeSolution { signature } => {
            Ok(AuthEvents::ReceivedSolution(ChallengeSolution {
                solution: signature,
            }))
        }
        _ => Err(Error::UnexpectedMessagePayload),
    }
}

pub async fn authenticate(props: &mut UserAgentConnection) -> Result<AuthPublicKey, Error> {
    let mut state = AuthStateMachine::new(AuthContext::new(props));

    loop {
        // `state` holds a mutable reference to `props` so we can't access it directly here
        let transport = state.context_mut().conn.transport.as_mut();
        let Some(payload) = transport.recv().await else {
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
    mut props: UserAgentConnection,
) -> Result<UserAgentSession, Error> {
    let _key = authenticate(&mut props).await?;
    let session = UserAgentSession::new(props);
    Ok(session)
}
