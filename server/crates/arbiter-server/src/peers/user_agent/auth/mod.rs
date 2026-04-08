use arbiter_crypto::authn;
use arbiter_proto::transport::Bi;
use tracing::error;

mod state;
use state::*;

use super::{AuthCredentials, UserAgentConnection};

#[derive(Debug, Clone)]
pub enum Inbound {
    AuthChallengeRequest {
        pubkey: authn::PublicKey,
        bootstrap_token: Option<String>,
    },
    AuthChallengeSolution {
        signature: Vec<u8>,
    },
}

#[derive(Debug)]
pub enum Error {
    UnregisteredPublicKey,
    InvalidChallengeSolution,
    InvalidBootstrapToken,
    Internal { details: String },
    Transport,
}

impl Error {
    fn internal(details: impl Into<String>) -> Self {
        Self::Internal {
            details: details.into(),
        }
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        error!(?e, "Database error");
        Self::internal("Database error")
    }
}

#[derive(Debug, Clone)]
pub enum Outbound {
    AuthChallenge { nonce: i32 },
    AuthSuccess,
}

fn parse_auth_event(payload: Inbound) -> AuthEvents {
    match payload {
        Inbound::AuthChallengeRequest {
            pubkey,
            bootstrap_token: None,
        } => AuthEvents::AuthRequest(ChallengeRequest { pubkey }),
        Inbound::AuthChallengeRequest {
            pubkey,
            bootstrap_token: Some(token),
        } => AuthEvents::BootstrapAuthRequest(BootstrapAuthRequest { pubkey, token }),
        Inbound::AuthChallengeSolution { signature } => {
            AuthEvents::ReceivedSolution(ChallengeSolution {
                solution: signature,
            })
        }
    }
}

pub async fn authenticate<T>(
    props: &mut UserAgentConnection,
    transport: T,
) -> Result<AuthCredentials, Error>
where
    T: Bi<Inbound, Result<Outbound, Error>> + Send,
{
    let mut state = AuthStateMachine::new(AuthContext::new(props, transport));

    loop {
        // `state` holds a mutable reference to `props` so we can't access it directly here
        let Some(payload) = state.context_mut().transport.recv().await else {
            return Err(Error::Transport);
        };

        match state.process_event(parse_auth_event(payload)).await {
            Ok(AuthStates::AuthOk(result)) => return Ok(result.clone()),
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
