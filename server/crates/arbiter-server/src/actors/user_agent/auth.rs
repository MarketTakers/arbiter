use arbiter_proto::proto::user_agent::{
    AuthChallengeRequest, AuthChallengeSolution, KeyType as ProtoKeyType, UserAgentRequest,
    user_agent_request::Payload as UserAgentRequestPayload,
};
use tracing::error;

use crate::actors::user_agent::{
    UserAgentConnection,
    auth::state::{AuthContext, AuthPublicKey, AuthStateMachine}, session::UserAgentSession,
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

fn parse_pubkey(key_type: ProtoKeyType, pubkey: Vec<u8>) -> Result<AuthPublicKey, Error> {
    match key_type {
        // UNSPECIFIED treated as Ed25519 for backward compatibility
        ProtoKeyType::Unspecified | ProtoKeyType::Ed25519 => {
            let pubkey_bytes = pubkey.as_array().ok_or(Error::InvalidClientPubkeyLength)?;
            let key = ed25519_dalek::VerifyingKey::from_bytes(pubkey_bytes)
                .map_err(|_| Error::InvalidAuthPubkeyEncoding)?;
            Ok(AuthPublicKey::Ed25519(key))
        }
        ProtoKeyType::EcdsaSecp256k1 => {
            // Public key is sent as 33-byte SEC1 compressed point
            let key = k256::ecdsa::VerifyingKey::from_sec1_bytes(&pubkey)
                .map_err(|_| Error::InvalidAuthPubkeyEncoding)?;
            Ok(AuthPublicKey::EcdsaSecp256k1(key))
        }
        ProtoKeyType::Rsa => {
            use rsa::pkcs8::DecodePublicKey as _;
            let key = rsa::RsaPublicKey::from_public_key_der(&pubkey)
                .map_err(|_| Error::InvalidAuthPubkeyEncoding)?;
            Ok(AuthPublicKey::Rsa(key))
        }
    }
}

fn parse_auth_event(payload: UserAgentRequestPayload) -> Result<AuthEvents, Error> {
    match payload {
        UserAgentRequestPayload::AuthChallengeRequest(AuthChallengeRequest {
            pubkey,
            bootstrap_token: None,
            key_type,
        }) => {
            let kt = ProtoKeyType::try_from(key_type).unwrap_or(ProtoKeyType::Unspecified);
            Ok(AuthEvents::AuthRequest(ChallengeRequest {
                pubkey: parse_pubkey(kt, pubkey)?,
            }))
        }
        UserAgentRequestPayload::AuthChallengeRequest(AuthChallengeRequest {
            pubkey,
            bootstrap_token: Some(token),
            key_type,
        }) => {
            let kt = ProtoKeyType::try_from(key_type).unwrap_or(ProtoKeyType::Unspecified);
            Ok(AuthEvents::BootstrapAuthRequest(BootstrapAuthRequest {
                pubkey: parse_pubkey(kt, pubkey)?,
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

pub async fn authenticate(props: &mut UserAgentConnection) -> Result<AuthPublicKey, Error> {
    let mut state = AuthStateMachine::new(AuthContext::new(props));

    loop {
        // `state` holds a mutable reference to `props` so we can't access it directly here
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

pub async fn authenticate_and_create(mut props: UserAgentConnection) -> Result<UserAgentSession, Error> {
    let _key = authenticate(&mut props).await?;
    let session = UserAgentSession::new(props);
    Ok(session)
}
