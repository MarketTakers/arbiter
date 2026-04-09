use arbiter_crypto::authn::{CLIENT_CONTEXT, SigningKey, format_challenge};
use arbiter_proto::{
    ClientMetadata,
    proto::{
        client::{
            ClientRequest,
            auth::{
                self as proto_auth, AuthChallenge, AuthChallengeRequest, AuthChallengeSolution,
                AuthResult, request::Payload as AuthRequestPayload,
                response::Payload as AuthResponsePayload,
            },
            client_request::Payload as ClientRequestPayload,
            client_response::Payload as ClientResponsePayload,
        },
        shared::ClientInfo as ProtoClientInfo,
    },
};

use crate::{
    storage::StorageError,
    transport::{ClientTransport, next_request_id},
};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Client approval denied by User Agent")]
    ApprovalDenied,

    #[error("Auth challenge was not returned by server")]
    MissingAuthChallenge,

    #[error("No User Agents online to approve client")]
    NoUserAgentsOnline,

    #[error("Signing key storage error")]
    Storage(#[from] StorageError),

    #[error("Unexpected auth response payload")]
    UnexpectedAuthResponse,
}

fn map_auth_result(code: i32) -> AuthError {
    match AuthResult::try_from(code).unwrap_or(AuthResult::Unspecified) {
        AuthResult::ApprovalDenied => AuthError::ApprovalDenied,
        AuthResult::NoUserAgentsOnline => AuthError::NoUserAgentsOnline,
        AuthResult::Unspecified
        | AuthResult::Success
        | AuthResult::InvalidKey
        | AuthResult::InvalidSignature
        | AuthResult::Internal => AuthError::UnexpectedAuthResponse,
    }
}

async fn send_auth_challenge_request(
    transport: &mut ClientTransport,
    metadata: ClientMetadata,
    key: &SigningKey,
) -> Result<(), AuthError> {
    transport
        .send(ClientRequest {
            request_id: next_request_id(),
            payload: Some(ClientRequestPayload::Auth(proto_auth::Request {
                payload: Some(AuthRequestPayload::ChallengeRequest(AuthChallengeRequest {
                    pubkey: key.public_key().to_bytes(),
                    client_info: Some(ProtoClientInfo {
                        name: metadata.name,
                        description: metadata.description,
                        version: metadata.version,
                    }),
                })),
            })),
        })
        .await
        .map_err(|_| AuthError::UnexpectedAuthResponse)
}

async fn receive_auth_challenge(
    transport: &mut ClientTransport,
) -> Result<AuthChallenge, AuthError> {
    let response = transport
        .recv()
        .await
        .map_err(|_| AuthError::MissingAuthChallenge)?;

    let payload = response.payload.ok_or(AuthError::MissingAuthChallenge)?;
    match payload {
        ClientResponsePayload::Auth(response) => match response.payload {
            Some(AuthResponsePayload::Challenge(challenge)) => Ok(challenge),
            Some(AuthResponsePayload::Result(result)) => Err(map_auth_result(result)),
            None => Err(AuthError::MissingAuthChallenge),
        },
        _ => Err(AuthError::UnexpectedAuthResponse),
    }
}

async fn send_auth_challenge_solution(
    transport: &mut ClientTransport,
    key: &SigningKey,
    challenge: AuthChallenge,
) -> Result<(), AuthError> {
    let challenge_payload = format_challenge(challenge.nonce, &challenge.pubkey);
    let signature = key
        .sign_message(&challenge_payload, CLIENT_CONTEXT)
        .map_err(|_| AuthError::UnexpectedAuthResponse)?
        .to_bytes();

    transport
        .send(ClientRequest {
            request_id: next_request_id(),
            payload: Some(ClientRequestPayload::Auth(proto_auth::Request {
                payload: Some(AuthRequestPayload::ChallengeSolution(
                    AuthChallengeSolution { signature },
                )),
            })),
        })
        .await
        .map_err(|_| AuthError::UnexpectedAuthResponse)
}

async fn receive_auth_confirmation(transport: &mut ClientTransport) -> Result<(), AuthError> {
    let response = transport
        .recv()
        .await
        .map_err(|_| AuthError::UnexpectedAuthResponse)?;

    let payload = response.payload.ok_or(AuthError::UnexpectedAuthResponse)?;
    match payload {
        ClientResponsePayload::Auth(response) => match response.payload {
            Some(AuthResponsePayload::Result(result))
                if AuthResult::try_from(result).ok() == Some(AuthResult::Success) =>
            {
                Ok(())
            }
            Some(AuthResponsePayload::Result(result)) => Err(map_auth_result(result)),
            _ => Err(AuthError::UnexpectedAuthResponse),
        },
        _ => Err(AuthError::UnexpectedAuthResponse),
    }
}

pub async fn authenticate(
    transport: &mut ClientTransport,
    metadata: ClientMetadata,
    key: &SigningKey,
) -> Result<(), AuthError> {
    send_auth_challenge_request(transport, metadata, key).await?;
    let challenge = receive_auth_challenge(transport).await?;
    send_auth_challenge_solution(transport, key, challenge).await?;
    receive_auth_confirmation(transport).await
}
