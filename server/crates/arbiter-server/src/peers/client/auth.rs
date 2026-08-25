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
        models::ProgramClientMetadata,
        schema::program_client,
    },
};
use arbiter_crypto::authn::{self, AuthChallenge, CLIENT_CONTEXT};
use arbiter_proto::{
    ClientMetadata,
    transport::{Bi, expect_message},
};

use diesel::{
    ExpressionMethods as _, OptionalExtension as _, QueryDsl as _, SelectableHelper as _,
    dsl::insert_into,
};
use diesel_async::RunQueryDsl as _;
use kameo::{actor::ActorRef, error::SendError};
use tracing::{error, warn};

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
    #[error("Client connection denied by operators")]
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

    conn.exclusive_transaction(async |conn| {
        let metadata_id = insert_into(client_metadata::table)
            .values((
                client_metadata::name.eq(&metadata.name),
                client_metadata::description.eq(&metadata.description),
                client_metadata::version.eq(&metadata.version),
            ))
            .returning(client_metadata::id)
            .get_result::<i32>(&mut *conn)
            .await?;

        let client_id = insert_into(program_client::table)
            .values((
                program_client::public_key.eq(pubkey.to_bytes()),
                program_client::metadata_id.eq(metadata_id),
            ))
            .on_conflict_do_nothing()
            .returning(program_client::id)
            .get_result::<i32>(&mut *conn)
            .await?;

        integrity::sign_entity(
            &mut *conn,
            vault,
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
    .await
}

/// Compares stored metadata against what a reconnecting client presents.
/// Metadata is frozen after initial operator approval and must not be silently
/// overwritten. Doing so would let an approved client forge its displayed
/// identity in later approval prompts. Drift is logged and ignored.
async fn check_metadata_drift(
    db: &db::DatabasePool,
    client_id: i32,
    presented: &ClientMetadata,
) -> Result<(), Error> {
    use crate::db::schema::client_metadata;

    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        Error::DatabasePoolUnavailable
    })?;

    let current: ProgramClientMetadata = program_client::table
        .find(client_id)
        .inner_join(client_metadata::table)
        .select(ProgramClientMetadata::as_select())
        .first(&mut conn)
        .await
        .map_err(|e| {
            error!(error = ?e, "Database error");
            Error::DatabaseOperationFailed
        })?;

    let changed = current.name != presented.name
        || current.description != presented.description
        || current.version != presented.version;

    if changed {
        warn!(
            client_id,
            stored_name = %current.name,
            presented_name = %presented.name,
            "reconnecting client presented different metadata; ignoring - metadata is frozen after operator approval"
        );
    }

    Ok(())
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
        Inbound::AuthChallengeRequest { .. } => None,
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
        check_metadata_drift(&props.db, id, &metadata).await?;
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
