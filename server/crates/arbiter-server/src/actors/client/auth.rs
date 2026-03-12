use arbiter_proto::{
    format_challenge,
    proto::client::{
        AuthChallenge, AuthChallengeSolution, ClientConnectError, ClientRequest, ClientResponse,
        client_connect_error::Code as ConnectErrorCode,
        client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
    },
    transport::expect_message,
};
use diesel::{
    ExpressionMethods as _, OptionalExtension as _, QueryDsl as _, dsl::insert_into, update,
};
use diesel_async::RunQueryDsl as _;
use ed25519_dalek::VerifyingKey;
use kameo::error::SendError;
use tracing::error;

use crate::{
    actors::{client::ClientConnection, router::{self, RequestClientApproval}},
    db::{self, schema::program_client},
};

use super::session::ClientSession;

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
    #[error("Invalid challenge solution")]
    InvalidChallengeSolution,
    #[error("Client approval request failed")]
    ApproveError(#[from] ApproveError),
    #[error("Internal error")]
    InternalError,
    #[error("Transport error")]
    Transport,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ApproveError {
    #[error("Internal error")]
    Internal,
    #[error("Client connection denied by user agents")]
    Denied,
    #[error("Upstream error: {0}")]
    Upstream(router::ApprovalError),
}

/// Atomically reads and increments the nonce for a known client.
/// Returns `None` if the pubkey is not registered.
async fn get_nonce(db: &db::DatabasePool, pubkey: &VerifyingKey) -> Result<Option<i32>, Error> {
    let pubkey_bytes = pubkey.as_bytes().to_vec();

    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;

    conn.exclusive_transaction(|conn| {
        let pubkey_bytes = pubkey_bytes.clone();
        Box::pin(async move {
            let Some(current_nonce) = program_client::table
                .filter(program_client::public_key.eq(&pubkey_bytes))
                .select(program_client::nonce)
                .first::<i32>(conn)
                .await
                .optional()?
            else {
                return Result::<_, diesel::result::Error>::Ok(None);
            };

            update(program_client::table)
                .filter(program_client::public_key.eq(&pubkey_bytes))
                .set(program_client::nonce.eq(current_nonce + 1))
                .execute(conn)
                .await?;

            Ok(Some(current_nonce))
        })
    })
    .await
    .map_err(|e| {
        error!(error = ?e, "Database error");
        Error::DatabaseOperationFailed
    })
}

async fn approve_new_client(
    actors: &crate::actors::GlobalActors,
    pubkey: VerifyingKey,
) -> Result<(), Error> {
    let result = actors
        .router
        .ask(RequestClientApproval { client_pubkey: pubkey })
        .await;

    match result {
        Ok(true) => Ok(()),
        Ok(false) => Err(Error::ApproveError(ApproveError::Denied)),
        Err(SendError::HandlerError(e)) => {
            error!(error = ?e, "Approval upstream error");
            Err(Error::ApproveError(ApproveError::Upstream(e)))
        }
        Err(e) => {
            error!(error = ?e, "Approval request to router failed");
            Err(Error::ApproveError(ApproveError::Internal))
        }
    }
}

async fn insert_client(db: &db::DatabasePool, pubkey: &VerifyingKey) -> Result<(), Error> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i32;

    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;

    insert_into(program_client::table)
        .values((
            program_client::public_key.eq(pubkey.as_bytes().to_vec()),
            program_client::nonce.eq(1), // pre-incremented; challenge uses 0
            program_client::created_at.eq(now),
            program_client::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to insert new client");
            Error::DatabaseOperationFailed
        })?;

    Ok(())
}

async fn challenge_client(
    props: &mut ClientConnection,
    pubkey: VerifyingKey,
    nonce: i32,
) -> Result<(), Error> {
    let challenge = AuthChallenge {
        pubkey: pubkey.as_bytes().to_vec(),
        nonce,
    };

    props
        .transport
        .send(Ok(ClientResponse {
            payload: Some(ClientResponsePayload::AuthChallenge(challenge.clone())),
        }))
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to send auth challenge");
            Error::Transport
        })?;

    let AuthChallengeSolution { signature } = expect_message(
        &mut *props.transport,
        |req: ClientRequest| match req.payload? {
            ClientRequestPayload::AuthChallengeSolution(s) => Some(s),
            _ => None,
        },
    )
    .await
    .map_err(|e| {
        error!(error = ?e, "Failed to receive challenge solution");
        Error::Transport
    })?;

    let formatted = format_challenge(nonce, &challenge.pubkey);
    let sig = signature.as_slice().try_into().map_err(|_| {
        error!("Invalid signature length");
        Error::InvalidChallengeSolution
    })?;

    pubkey.verify_strict(&formatted, &sig).map_err(|_| {
        error!("Challenge solution verification failed");
        Error::InvalidChallengeSolution
    })?;

    Ok(())
}

fn connect_error_code(err: &Error) -> ConnectErrorCode {
    match err {
        Error::ApproveError(ApproveError::Denied) => ConnectErrorCode::ApprovalDenied,
        Error::ApproveError(ApproveError::Upstream(router::ApprovalError::NoUserAgentsConnected)) => {
            ConnectErrorCode::NoUserAgentsOnline
        }
        _ => ConnectErrorCode::Unknown,
    }
}

async fn authenticate(props: &mut ClientConnection) -> Result<VerifyingKey, Error> {
    let Some(ClientRequest {
        payload: Some(ClientRequestPayload::AuthChallengeRequest(challenge)),
    }) = props.transport.recv().await
    else {
        return Err(Error::Transport);
    };

    let pubkey_bytes = challenge
        .pubkey
        .as_array()
        .ok_or(Error::InvalidClientPubkeyLength)?;
    let pubkey =
        VerifyingKey::from_bytes(pubkey_bytes).map_err(|_| Error::InvalidAuthPubkeyEncoding)?;

    let nonce = match get_nonce(&props.db, &pubkey).await? {
        Some(nonce) => nonce,
        None => {
            approve_new_client(&props.actors, pubkey).await?;
            insert_client(&props.db, &pubkey).await?;
            0
        }
    };

    challenge_client(props, pubkey, nonce).await?;

    Ok(pubkey)
}

pub async fn authenticate_and_create(mut props: ClientConnection) -> Result<ClientSession, Error> {
    match authenticate(&mut props).await {
        Ok(pubkey) => Ok(ClientSession::new(props, pubkey)),
        Err(err) => {
            let code = connect_error_code(&err);
            let _ = props
                .transport
                .send(Ok(ClientResponse {
                    payload: Some(ClientResponsePayload::ClientConnectError(
                        ClientConnectError { code: code.into() },
                    )),
                }))
                .await;
            Err(err)
        }
    }
}
