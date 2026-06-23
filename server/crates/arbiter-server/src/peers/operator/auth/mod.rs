use super::{Credentials, OperatorConnection};
use arbiter_crypto::authn::{self, AuthChallenge};
use arbiter_proto::transport::Bi;

use state::{
    AuthContext, AuthError, AuthEvents, AuthStateMachine, AuthStates, ChallengeRequest,
    ChallengeSolution,
};
use tracing::error;

mod state;

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
    AuthChallenge { challenge: AuthChallenge },
    AuthSuccess,
}

fn parse_auth_event(payload: Inbound) -> AuthEvents {
    match payload {
        Inbound::AuthChallengeRequest {
            pubkey,
            bootstrap_token,
        } => AuthEvents::AuthRequest(ChallengeRequest {
            pubkey,
            bootstrap_token,
        }),
        Inbound::AuthChallengeSolution { signature } => {
            AuthEvents::ReceivedSolution(ChallengeSolution {
                solution: signature,
            })
        }
    }
}

pub async fn authenticate<T>(
    props: &mut OperatorConnection,
    transport: &mut T,
) -> Result<Credentials, Error>
where
    T: Bi<Inbound, Result<Outbound, Error>> + Send + ?Sized,
{
    let mut state = AuthStateMachine::new(AuthContext::new(props, transport));

    loop {
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
