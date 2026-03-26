use arbiter_proto::{
    format_challenge,
    transport::{Bi, expect_message},
};
use diesel::{
    ExpressionMethods as _, OptionalExtension as _, QueryDsl as _, dsl::insert_into, update,
};
use diesel_async::RunQueryDsl as _;
use ed25519_dalek::{Signature, VerifyingKey};
use kameo::error::SendError;
use tracing::error;

use crate::{
    actors::{
        client::ClientConnection,
        router::{self, RequestClientApproval},
    },
    db::{self, schema::program_client},
};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Database pool unavailable")]
    DatabasePoolUnavailable,
    #[error("Database operation failed")]
    DatabaseOperationFailed,
    #[error("Invalid challenge solution")]
    InvalidChallengeSolution,
    #[error("Client approval request failed")]
    ApproveError(#[from] ApproveError),
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

#[derive(Debug, Clone)]
pub enum Inbound {
    AuthChallengeRequest { pubkey: VerifyingKey },
    AuthChallengeSolution { signature: Signature },
}

#[derive(Debug, Clone)]
pub enum Outbound {
    AuthChallenge { pubkey: VerifyingKey, nonce: i32 },
    AuthSuccess,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedClient {
    pub pubkey: VerifyingKey,
    pub client_id: i32,
}

/// Atomically reads and increments the nonce for a known client.
/// Returns `None` if the pubkey is not registered.
async fn get_nonce(
    db: &db::DatabasePool,
    pubkey: &VerifyingKey,
) -> Result<Option<(/* client_id */ i32, /* nonce */ i32)>, Error> {
    let pubkey_bytes = pubkey.as_bytes();

    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;

    conn.exclusive_transaction(|conn| {
        Box::pin(async move {
            let Some((client_id, current_nonce)) = program_client::table
                .filter(program_client::public_key.eq(&pubkey_bytes))
                .select((program_client::id, program_client::nonce))
                .first::<(i32, i32)>(conn)
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

            Ok(Some((client_id, current_nonce)))
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
        .ask(RequestClientApproval {
            client_pubkey: pubkey,
        })
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

enum InsertClientResult {
    Inserted,
    AlreadyExists,
}

async fn insert_client(
    db: &db::DatabasePool,
    pubkey: &VerifyingKey,
) -> Result<InsertClientResult, Error> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i32;

    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;

    match insert_into(program_client::table)
        .values((
            program_client::public_key.eq(pubkey.as_bytes().to_vec()),
            program_client::nonce.eq(1), // pre-incremented; challenge uses 0
            program_client::created_at.eq(now),
            program_client::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .await
    {
        Ok(_) => {}
        Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        )) => return Ok(InsertClientResult::AlreadyExists),
        Err(e) => {
            error!(error = ?e, "Failed to insert new client");
            return Err(Error::DatabaseOperationFailed);
        }
    }

    let client_id = program_client::table
        .filter(program_client::public_key.eq(pubkey.as_bytes().to_vec()))
        .order(program_client::id.desc())
        .select(program_client::id)
        .first::<i32>(&mut conn)
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to load inserted client id");
            Error::DatabaseOperationFailed
        })?;

    let _ = client_id;
    Ok(InsertClientResult::Inserted)
}

async fn challenge_client<T>(
    transport: &mut T,
    pubkey: VerifyingKey,
    nonce: i32,
) -> Result<(), Error>
where
    T: Bi<Inbound, Result<Outbound, Error>> + ?Sized,
{
    transport
        .send(Ok(Outbound::AuthChallenge { pubkey, nonce }))
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to send auth challenge");
            Error::Transport
        })?;

    let signature = expect_message(transport, |req: Inbound| match req {
        Inbound::AuthChallengeSolution { signature } => Some(signature),
        _ => None,
    })
    .await
    .map_err(|e| {
        error!(error = ?e, "Failed to receive challenge solution");
        Error::Transport
    })?;

    let formatted = format_challenge(nonce, pubkey.as_bytes());

    pubkey.verify_strict(&formatted, &signature).map_err(|_| {
        error!("Challenge solution verification failed");
        Error::InvalidChallengeSolution
    })?;

    Ok(())
}

pub async fn authenticate<T>(
    props: &mut ClientConnection,
    transport: &mut T,
) -> Result<AuthenticatedClient, Error>
where
    T: Bi<Inbound, Result<Outbound, Error>> + Send + ?Sized,
{
    let Some(Inbound::AuthChallengeRequest { pubkey }) = transport.recv().await else {
        return Err(Error::Transport);
    };

    let (client_id, nonce) = match get_nonce(&props.db, &pubkey).await? {
        Some(client_nonce) => client_nonce,
        None => {
            approve_new_client(&props.actors, pubkey).await?;
            match insert_client(&props.db, &pubkey).await? {
                InsertClientResult::Inserted => match get_nonce(&props.db, &pubkey).await? {
                    Some((client_id, _)) => (client_id, 0),
                    None => return Err(Error::DatabaseOperationFailed),
                },
                InsertClientResult::AlreadyExists => match get_nonce(&props.db, &pubkey).await? {
                    Some((client_id, nonce)) => (client_id, nonce),
                    None => return Err(Error::DatabaseOperationFailed),
                },
            }
        }
    };

    challenge_client(transport, pubkey, nonce).await?;
    transport
        .send(Ok(Outbound::AuthSuccess))
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to send auth success");
            Error::Transport
        })?;

    Ok(AuthenticatedClient { pubkey, client_id })
}
