use super::{ClientConnection, ClientCredentials, ClientProfile};
use crate::{
    actors::{
        GlobalActors,
        flow_coordinator::{self, RequestClientApproval},
        vault::Vault,
    },
    crypto::integrity::{self, AttestationStatus},
    db::{
        self,
        models::{ProgramClientMetadata, SqliteTimestamp},
        schema::program_client,
    },
};
use arbiter_crypto::authn::{self, AuthChallenge, CLIENT_CONTEXT};
use arbiter_proto::{
    ClientMetadata,
    transport::{Bi, expect_message},
};

use chrono::Utc;
use diesel::{
    ExpressionMethods as _, OptionalExtension as _, QueryDsl as _, SelectableHelper as _,
    dsl::insert_into, update,
};
use diesel_async::RunQueryDsl as _;
use kameo::{actor::ActorRef, error::SendError};
use tracing::error;

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
        pubkey: authn::PublicKey,
        metadata: ClientMetadata,
    },
    AuthChallengeSolution {
        signature: authn::Signature,
    },
}

#[derive(Debug, Clone)]
pub enum Outbound {
    AuthChallenge { challenge: AuthChallenge },
    AuthSuccess,
}

async fn get_client_id(
    db: &db::DatabasePool,
    pubkey: &authn::PublicKey,
) -> Result<Option<i32>, Error> {
    let pubkey_bytes = pubkey.to_bytes();
    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;
    program_client::table
        .filter(program_client::public_key.eq(&pubkey_bytes))
        .select(program_client::id)
        .first::<i32>(&mut conn)
        .await
        .optional()
        .map_err(|e| {
            error!(error = ?e, "Database error");
            Error::DatabaseOperationFailed
        })
}

async fn verify_integrity(
    db: &db::DatabasePool,
    vault: &ActorRef<Vault>,
    pubkey: &authn::PublicKey,
) -> Result<(), Error> {
    let mut db_conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;

    let id = get_client_id(db, pubkey).await?.ok_or_else(|| {
        error!("Client not found during integrity verification");
        Error::DatabaseOperationFailed
    })?;

    let attestation = integrity::verify_entity(
        &mut db_conn,
        vault,
        &ClientCredentials {
            pubkey: pubkey.clone(),
        },
        id,
    )
    .await
    .map_err(|e| {
        error!(?e, "Integrity verification failed");
        Error::IntegrityCheckFailed
    })?;

    if attestation != AttestationStatus::Attested {
        error!("Integrity attestation unavailable for client {id}");
        return Err(Error::IntegrityCheckFailed);
    }

    Ok(())
}

async fn approve_new_client(actors: &GlobalActors, profile: ClientProfile) -> Result<(), Error> {
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
    vault: &ActorRef<Vault>,
    pubkey: &authn::PublicKey,
    metadata: &ClientMetadata,
) -> Result<i32, Error> {
    use crate::db::schema::client_metadata;

    let pubkey = pubkey.clone();
    let metadata = metadata.clone();

    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;

    conn.exclusive_transaction(|conn| {
        let vault = vault.clone();
        let pubkey = pubkey.clone();
        Box::pin(async move {
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
                    program_client::public_key.eq(pubkey.to_bytes()),
                    program_client::metadata_id.eq(metadata_id),
                ))
                .on_conflict_do_nothing()
                .returning(program_client::id)
                .get_result::<i32>(conn)
                .await?;

            integrity::sign_entity(
                conn,
                &vault,
                &ClientCredentials {
                    pubkey: pubkey.clone(),
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
    pubkey: authn::PublicKey,
    challenge: AuthChallenge,
) -> Result<(), Error>
where
    T: Bi<Inbound, Result<Outbound, Error>> + ?Sized,
{
    transport
        .send(Ok(Outbound::AuthChallenge {
            challenge: challenge.clone(),
        }))
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

    if !pubkey.verify(&challenge, CLIENT_CONTEXT, &signature) {
        error!("Challenge solution verification failed");
        return Err(Error::InvalidChallengeSolution);
    }

    Ok(())
}

pub async fn authenticate<T>(props: &mut ClientConnection, transport: &mut T) -> Result<i32, Error>
where
    T: Bi<Inbound, Result<Outbound, Error>> + Send + ?Sized,
{
    let Some(Inbound::AuthChallengeRequest { pubkey, metadata }) = transport.recv().await else {
        return Err(Error::Transport);
    };

    let client_id = if let Some(id) = get_client_id(&props.db, &pubkey).await? {
        verify_integrity(&props.db, &props.actors.vault, &pubkey).await?;
        id
    } else {
        approve_new_client(
            &props.actors,
            ClientProfile {
                pubkey: pubkey.clone(),
                metadata: metadata.clone(),
            },
        )
        .await?;
        insert_client(&props.db, &props.actors.vault, &pubkey, &metadata).await?
    };

    sync_client_metadata(&props.db, client_id, &metadata).await?;

    let challenge = AuthChallenge::generate(&mut rand::rng());
    challenge_client(transport, pubkey, challenge).await?;

    transport
        .send(Ok(Outbound::AuthSuccess))
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to send auth success");
            Error::Transport
        })?;

    Ok(client_id)
}
