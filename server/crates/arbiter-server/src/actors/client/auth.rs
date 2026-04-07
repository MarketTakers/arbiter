use arbiter_proto::{
    ClientMetadata, format_challenge,
    transport::{Bi, expect_message},
};
use chrono::Utc;
use diesel::{
    ExpressionMethods as _, OptionalExtension as _, QueryDsl as _, SelectableHelper as _,
    dsl::insert_into, update,
};
use diesel_async::RunQueryDsl as _;
use ed25519_dalek::{Signature, VerifyingKey};
use kameo::{actor::ActorRef, error::SendError};
use tracing::error;

use crate::{
    actors::{
        client::{ClientConnection, ClientCredentials, ClientProfile},
        flow_coordinator::{self, RequestClientApproval},
        keyholder::KeyHolder,
    },
    crypto::integrity::{self},
    db::{
        self,
        models::{ProgramClientMetadata, SqliteTimestamp},
        schema::program_client,
    },
};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Database pool unavailable")]
    DatabasePoolUnavailable,
    #[error("Database operation failed")]
    DatabaseOperationFailed,
    #[error("Integrity check failed")]
    IntegrityCheckFailed,
    #[error("Invalid challenge solution")]
    InvalidChallengeSolution,
    #[error("Client approval request failed")]
    ApproveError(#[from] ApproveError),
    #[error("Transport error")]
    Transport,
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        error!(?e, "Database error");
        Self::DatabaseOperationFailed
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ApproveError {
    #[error("Internal error")]
    Internal,
    #[error("Client connection denied by user agents")]
    Denied,
    #[error("Upstream error: {0}")]
    Upstream(flow_coordinator::ApprovalError),
}

#[derive(Debug, Clone)]
pub enum Inbound {
    AuthChallengeRequest {
        pubkey: VerifyingKey,
        metadata: ClientMetadata,
    },
    AuthChallengeSolution {
        signature: Signature,
    },
}

#[derive(Debug, Clone)]
pub enum Outbound {
    AuthChallenge { pubkey: VerifyingKey, nonce: i32 },
    AuthSuccess,
}

/// Returns the current nonce and client ID for a registered client.
/// Returns `None` if the pubkey is not registered.
async fn get_current_nonce_and_id(
    db: &db::DatabasePool,
    pubkey: &VerifyingKey,
) -> Result<Option<(i32, i32)>, Error> {
    let pubkey_bytes = pubkey.as_bytes().to_vec();
    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;
    program_client::table
        .filter(program_client::public_key.eq(&pubkey_bytes))
        .select((program_client::id, program_client::nonce))
        .first::<(i32, i32)>(&mut conn)
        .await
        .optional()
        .map_err(|e| {
            error!(error = ?e, "Database error");
            Error::DatabaseOperationFailed
        })
}

async fn verify_integrity(
    db: &db::DatabasePool,
    keyholder: &ActorRef<KeyHolder>,
    pubkey: &VerifyingKey,
) -> Result<(), Error> {
    let mut db_conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;

    let (id, nonce) = get_current_nonce_and_id(db, pubkey).await?.ok_or_else(|| {
        error!("Client not found during integrity verification");
        Error::DatabaseOperationFailed
    })?;

    integrity::verify_entity(
        &mut db_conn,
        keyholder,
        &ClientCredentials {
            pubkey: *pubkey,
            nonce,
        },
        id,
    )
    .await
    .map_err(|e| {
        error!(?e, "Integrity verification failed");
        Error::IntegrityCheckFailed
    })?;

    Ok(())
}

/// Atomically increments the nonce and re-signs the integrity envelope.
/// Returns the new nonce, which is used as the challenge nonce.
async fn create_nonce(
    db: &db::DatabasePool,
    keyholder: &ActorRef<KeyHolder>,
    pubkey: &VerifyingKey,
) -> Result<i32, Error> {
    let pubkey_bytes = pubkey.as_bytes().to_vec();

    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;

    conn.exclusive_transaction(|conn| {
        let keyholder = keyholder.clone();
        Box::pin(async move {
            let (id, new_nonce): (i32, i32) = update(program_client::table)
                .filter(program_client::public_key.eq(&pubkey_bytes))
                .set(program_client::nonce.eq(program_client::nonce + 1))
                .returning((program_client::id, program_client::nonce))
                .get_result(conn)
                .await?;

            integrity::sign_entity(
                conn,
                &keyholder,
                &ClientCredentials {
                    pubkey: *pubkey,
                    nonce: new_nonce,
                },
                id,
            )
            .await
            .map_err(|e| {
                error!(?e, "Integrity sign failed after nonce update");
                Error::DatabaseOperationFailed
            })?;

            Ok(new_nonce)
        })
    })
    .await
}

async fn approve_new_client(
    actors: &crate::actors::GlobalActors,
    profile: ClientProfile,
) -> Result<(), Error> {
    let result = actors
        .flow_coordinator
        .ask(RequestClientApproval { client: profile })
        .await;

    match result {
        Ok(true) => Ok(()),
        Ok(false) => Err(Error::ApproveError(ApproveError::Denied)),
        Err(SendError::HandlerError(e)) => {
            error!(error = ?e, "Approval upstream error");
            Err(Error::ApproveError(ApproveError::Upstream(e)))
        }
        Err(e) => {
            error!(error = ?e, "Approval request to flow coordinator failed");
            Err(Error::ApproveError(ApproveError::Internal))
        }
    }
}

async fn insert_client(
    db: &db::DatabasePool,
    keyholder: &ActorRef<KeyHolder>,
    pubkey: &VerifyingKey,
    metadata: &ClientMetadata,
) -> Result<i32, Error> {
    use crate::db::schema::{client_metadata, program_client};
    let metadata = metadata.clone();

    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;

    conn.exclusive_transaction(|conn| {
        let keyholder = keyholder.clone();
        Box::pin(async move {
            const NONCE_START: i32 = 1;

            let metadata_id = insert_into(client_metadata::table)
                .values((
                    client_metadata::name.eq(&metadata.name),
                    client_metadata::description.eq(&metadata.description),
                    client_metadata::version.eq(&metadata.version),
                ))
                .returning(client_metadata::id)
                .get_result::<i32>(conn)
                .await?;

            let client_id = insert_into(program_client::table)
                .values((
                    program_client::public_key.eq(pubkey.as_bytes().to_vec()),
                    program_client::metadata_id.eq(metadata_id),
                    program_client::nonce.eq(NONCE_START),
                ))
                .on_conflict_do_nothing()
                .returning(program_client::id)
                .get_result::<i32>(conn)
                .await?;

            integrity::sign_entity(
                conn,
                &keyholder,
                &ClientCredentials {
                    pubkey: *pubkey,
                    nonce: NONCE_START,
                },
                client_id,
            )
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to sign integrity tag for new client key");
                Error::DatabaseOperationFailed
            })?;

            Ok(client_id)
        })
    })
    .await
}

async fn sync_client_metadata(
    db: &db::DatabasePool,
    client_id: i32,
    metadata: &ClientMetadata,
) -> Result<(), Error> {
    use crate::db::schema::{client_metadata, client_metadata_history};

    let now = SqliteTimestamp(Utc::now());

    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;

    conn.exclusive_transaction(|conn| {
        let metadata = metadata.clone();
        Box::pin(async move {
            let (current_metadata_id, current): (i32, ProgramClientMetadata) =
                program_client::table
                    .find(client_id)
                    .inner_join(client_metadata::table)
                    .select((
                        program_client::metadata_id,
                        ProgramClientMetadata::as_select(),
                    ))
                    .first(conn)
                    .await?;

            let unchanged = current.name == metadata.name
                && current.description == metadata.description
                && current.version == metadata.version;
            if unchanged {
                return Ok(());
            }

            insert_into(client_metadata_history::table)
                .values((
                    client_metadata_history::metadata_id.eq(current_metadata_id),
                    client_metadata_history::client_id.eq(client_id),
                ))
                .execute(conn)
                .await?;

            let metadata_id = insert_into(client_metadata::table)
                .values((
                    client_metadata::name.eq(&metadata.name),
                    client_metadata::description.eq(&metadata.description),
                    client_metadata::version.eq(&metadata.version),
                ))
                .returning(client_metadata::id)
                .get_result::<i32>(conn)
                .await?;

            update(program_client::table.find(client_id))
                .set((
                    program_client::metadata_id.eq(metadata_id),
                    program_client::updated_at.eq(now),
                ))
                .execute(conn)
                .await?;

            Ok::<(), diesel::result::Error>(())
        })
    })
    .await
    .map_err(|e| {
        error!(error = ?e, "Database error");
        Error::DatabaseOperationFailed
    })
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

pub async fn authenticate<T>(props: &mut ClientConnection, transport: &mut T) -> Result<i32, Error>
where
    T: Bi<Inbound, Result<Outbound, Error>> + Send + ?Sized,
{
    let Some(Inbound::AuthChallengeRequest { pubkey, metadata }) = transport.recv().await else {
        return Err(Error::Transport);
    };

    let client_id = match get_current_nonce_and_id(&props.db, &pubkey).await? {
        Some((id, _)) => {
            verify_integrity(&props.db, &props.actors.key_holder, &pubkey).await?;
            id
        }
        None => {
            approve_new_client(
                &props.actors,
                ClientProfile {
                    pubkey,
                    metadata: metadata.clone(),
                },
            )
            .await?;
            insert_client(&props.db, &props.actors.key_holder, &pubkey, &metadata).await?
        }
    };

    sync_client_metadata(&props.db, client_id, &metadata).await?;
    let challenge_nonce = create_nonce(&props.db, &props.actors.key_holder, &pubkey).await?;
    challenge_client(transport, pubkey, challenge_nonce).await?;

    transport
        .send(Ok(Outbound::AuthSuccess))
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to send auth success");
            Error::Transport
        })?;

    Ok(client_id)
}
