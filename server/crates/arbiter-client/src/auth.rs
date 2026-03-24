use arbiter_proto::{
    format_challenge,
    proto::client::{
        AuthChallengeRequest, AuthChallengeSolution, AuthResult, ClientRequest,
        client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
    },
};
use ed25519_dalek::Signer as _;
use terrors::OneOf;

use crate::{
    errors::{
        ConnectError, MissingAuthChallengeError, UnexpectedAuthResponseError, map_auth_code_error,
    },
    transport::{ClientTransport, next_request_id},
};

async fn send_auth_challenge_request(
    transport: &mut ClientTransport,
    key: &ed25519_dalek::SigningKey,
) -> std::result::Result<(), ConnectError> {
    transport
        .send(ClientRequest {
            request_id: next_request_id(),
            payload: Some(ClientRequestPayload::AuthChallengeRequest(
                AuthChallengeRequest {
                    pubkey: key.verifying_key().to_bytes().to_vec(),
                },
            )),
        })
        .await
        .map_err(|_| OneOf::new(UnexpectedAuthResponseError))
}

async fn receive_auth_challenge(
    transport: &mut ClientTransport,
) -> std::result::Result<arbiter_proto::proto::client::AuthChallenge, ConnectError> {
    let response = transport
        .recv()
        .await
        .map_err(|_| OneOf::new(MissingAuthChallengeError))?;

    let payload = response
        .payload
        .ok_or_else(|| OneOf::new(MissingAuthChallengeError))?;
    match payload {
        ClientResponsePayload::AuthChallenge(challenge) => Ok(challenge),
        ClientResponsePayload::AuthResult(result) => Err(map_auth_code_error(result)),
        _ => Err(OneOf::new(UnexpectedAuthResponseError)),
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
        .map_err(|_| OneOf::new(UnexpectedAuthResponseError))
}

async fn receive_auth_confirmation(
    transport: &mut ClientTransport,
) -> std::result::Result<(), ConnectError> {
    let response = transport
        .recv()
        .await
        .map_err(|_| OneOf::new(UnexpectedAuthResponseError))?;

    let payload = response
        .payload
        .ok_or_else(|| OneOf::new(UnexpectedAuthResponseError))?;
    match payload {
        ClientResponsePayload::AuthResult(result)
            if AuthResult::try_from(result).ok() == Some(AuthResult::Success) =>
        {
            Ok(())
        }
        ClientResponsePayload::AuthResult(result) => Err(map_auth_code_error(result)),
        _ => Err(OneOf::new(UnexpectedAuthResponseError)),
    }
}

pub(crate) async fn authenticate(
    transport: &mut ClientTransport,
    key: &ed25519_dalek::SigningKey,
) -> std::result::Result<(), ConnectError> {
    send_auth_challenge_request(transport, key).await?;
    let challenge = receive_auth_challenge(transport).await?;
    send_auth_challenge_solution(transport, key, challenge).await?;
    receive_auth_confirmation(transport).await
}
