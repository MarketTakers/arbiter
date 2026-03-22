use arbiter_proto::{
    ClientMetadata, format_challenge,
    proto::client::{
        AuthChallengeRequest, AuthChallengeSolution, AuthResult, ClientInfo as ProtoClientInfo,
        ClientRequest, client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
    },
};
use ed25519_dalek::Signer as _;

use crate::{
    storage::StorageError,
    transport::{ClientTransport, next_request_id},
};

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("Could not establish connection")]
    Connection(#[from] tonic::transport::Error),

    #[error("Invalid server URI")]
    InvalidUri(#[from] http::uri::InvalidUri),

    #[error("Invalid CA certificate")]
    InvalidCaCert(#[from] webpki::Error),

    #[error("gRPC error")]
    Grpc(#[from] tonic::Status),

    #[error("Auth challenge was not returned by server")]
    MissingAuthChallenge,

    #[error("Client approval denied by User Agent")]
    ApprovalDenied,

    #[error("No User Agents online to approve client")]
    NoUserAgentsOnline,

    #[error("Unexpected auth response payload")]
    UnexpectedAuthResponse,

    #[error("Signing key storage error")]
    Storage(#[from] StorageError),
}

fn map_auth_result(code: i32) -> ConnectError {
    match AuthResult::try_from(code).unwrap_or(AuthResult::Unspecified) {
        AuthResult::ApprovalDenied => ConnectError::ApprovalDenied,
        AuthResult::NoUserAgentsOnline => ConnectError::NoUserAgentsOnline,
        AuthResult::Unspecified
        | AuthResult::Success
        | AuthResult::InvalidKey
        | AuthResult::InvalidSignature
        | AuthResult::Internal => ConnectError::UnexpectedAuthResponse,
    }
}

async fn send_auth_challenge_request(
    transport: &mut ClientTransport,
    metadata: ClientMetadata,
    key: &ed25519_dalek::SigningKey,
) -> std::result::Result<(), ConnectError> {
    transport
        .send(ClientRequest {
            request_id: next_request_id(),
            payload: Some(ClientRequestPayload::AuthChallengeRequest(
                AuthChallengeRequest {
                    pubkey: key.verifying_key().to_bytes().to_vec(),
                    client_info: Some(ProtoClientInfo {
                        name: metadata.name,
                        description: metadata.description,
                        version: metadata.version,
                    }),
                },
            )),
        })
        .await
        .map_err(|_| ConnectError::UnexpectedAuthResponse)
}

async fn receive_auth_challenge(
    transport: &mut ClientTransport,
) -> std::result::Result<arbiter_proto::proto::client::AuthChallenge, ConnectError> {
    let response = transport
        .recv()
        .await
        .map_err(|_| ConnectError::MissingAuthChallenge)?;

    let payload = response.payload.ok_or(ConnectError::MissingAuthChallenge)?;
    match payload {
        ClientResponsePayload::AuthChallenge(challenge) => Ok(challenge),
        ClientResponsePayload::AuthResult(result) => Err(map_auth_result(result)),
        _ => Err(ConnectError::UnexpectedAuthResponse),
    }
}

async fn send_auth_challenge_solution(
    transport: &mut ClientTransport,
    key: &ed25519_dalek::SigningKey,
    challenge: arbiter_proto::proto::client::AuthChallenge,
) -> std::result::Result<(), ConnectError> {
    let challenge_payload = format_challenge(challenge.nonce, &challenge.pubkey);
    let signature = key.sign(&challenge_payload).to_bytes().to_vec();

    transport
        .send(ClientRequest {
            request_id: next_request_id(),
            payload: Some(ClientRequestPayload::AuthChallengeSolution(
                AuthChallengeSolution { signature },
            )),
        })
        .await
        .map_err(|_| ConnectError::UnexpectedAuthResponse)
}

async fn receive_auth_confirmation(
    transport: &mut ClientTransport,
) -> std::result::Result<(), ConnectError> {
    let response = transport
        .recv()
        .await
        .map_err(|_| ConnectError::UnexpectedAuthResponse)?;

    let payload = response
        .payload
        .ok_or(ConnectError::UnexpectedAuthResponse)?;
    match payload {
        ClientResponsePayload::AuthResult(result)
            if AuthResult::try_from(result).ok() == Some(AuthResult::Success) =>
        {
            Ok(())
        }
        ClientResponsePayload::AuthResult(result) => Err(map_auth_result(result)),
        _ => Err(ConnectError::UnexpectedAuthResponse),
    }
}

pub(crate) async fn authenticate(
    transport: &mut ClientTransport,
    metadata: ClientMetadata,
    key: &ed25519_dalek::SigningKey,
) -> std::result::Result<(), ConnectError> {
    send_auth_challenge_request(transport, metadata, key).await?;
    let challenge = receive_auth_challenge(transport).await?;
    send_auth_challenge_solution(transport, key, challenge).await?;
    receive_auth_confirmation(transport).await
}
