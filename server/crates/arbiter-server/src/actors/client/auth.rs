use arbiter_crypto::authn::{self, CLIENT_CONTEXT};
use arbiter_proto::{
    ClientMetadata,
    proto::client::auth::{AuthChallenge as ProtoAuthChallenge, AuthResult as ProtoAuthResult},
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

use crate::{
    actors::{
        client::{ClientConnection, ClientCredentials, ClientProfile},
        flow_coordinator::{self, RequestClientApproval},
        keyholder::KeyHolder,
    },
    crypto::integrity::{self, AttestationStatus},
    db::{
        self,
        models::{ProgramClientMetadata, SqliteTimestamp},
        schema::program_client,
    },
};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ClientAuthError {
    #[error("Client approval request failed")]
    ApproveError(#[from] ApproveError),

    #[error("Database operation failed")]
    DatabaseOperationFailed,

    #[error("Database pool unavailable")]
    DatabasePoolUnavailable,

    #[error("Integrity check failed")]
    IntegrityCheckFailed,

    #[error("Invalid challenge solution")]
    InvalidChallengeSolution,

    #[error("Transport error")]
    Transport,
}

impl From<diesel::result::Error> for ClientAuthError {
    fn from(e: diesel::result::Error) -> Self {
        error!(?e, "Database error");
        Self::DatabaseOperationFailed
    }
}

impl From<ClientAuthError> for arbiter_proto::proto::client::auth::AuthResult {
    fn from(value: ClientAuthError) -> Self {
        match value {
            ClientAuthError::ApproveError(e) => match e {
                ApproveError::Denied => Self::ApprovalDenied,
                ApproveError::Internal => Self::Internal,
                ApproveError::Upstream(flow_coordinator::ApprovalError::NoUserAgentsConnected) => {
                    Self::NoUserAgentsOnline
                } // ApproveError::Upstream(_) => Self::Internal,
            },
            ClientAuthError::DatabaseOperationFailed
            | ClientAuthError::DatabasePoolUnavailable
            | ClientAuthError::IntegrityCheckFailed
            | ClientAuthError::Transport => Self::Internal,
            ClientAuthError::InvalidChallengeSolution => Self::InvalidSignature,
        }
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ApproveError {
    #[error("Client connection denied by user agents")]
    Denied,

    #[error("Internal error")]
    Internal,

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
    AuthChallenge {
        pubkey: authn::PublicKey,
        nonce: i32,
    },
    AuthSuccess,
}

impl From<Outbound> for arbiter_proto::proto::client::auth::response::Payload {
    fn from(value: Outbound) -> Self {
        match value {
            Outbound::AuthChallenge { pubkey, nonce } => Self::Challenge(ProtoAuthChallenge {
                pubkey: pubkey.to_bytes(),
                nonce,
            }),
            Outbound::AuthSuccess => Self::Result(ProtoAuthResult::Success.into()),
        }
    }
}

/// Returns the current nonce and client ID for a registered client.
/// Returns `None` if the pubkey is not registered.
async fn get_current_nonce_and_id(
    db: &db::DatabasePool,
    pubkey: &authn::PublicKey,
) -> Result<Option<(i32, i32)>, ClientAuthError> {
    let pubkey_bytes = pubkey.to_bytes();
    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        ClientAuthError::DatabasePoolUnavailable
    })?;
    program_client::table
        .filter(program_client::public_key.eq(&pubkey_bytes))
        .select((program_client::id, program_client::nonce))
        .first::<(i32, i32)>(&mut conn)
        .await
        .optional()
        .map_err(|e| {
            error!(error = ?e, "Database error");
            ClientAuthError::DatabaseOperationFailed
        })
}

async fn verify_integrity(
    db: &db::DatabasePool,
    keyholder: &ActorRef<KeyHolder>,
    pubkey: &authn::PublicKey,
) -> Result<(), ClientAuthError> {
    let mut db_conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        ClientAuthError::DatabasePoolUnavailable
    })?;

    let (id, nonce) = get_current_nonce_and_id(db, pubkey).await?.ok_or_else(|| {
        error!("Client not found during integrity verification");
        ClientAuthError::DatabaseOperationFailed
    })?;

    let attestation = integrity::verify_entity(
        &mut db_conn,
        keyholder,
        &ClientCredentials {
            pubkey: pubkey.clone(),
            nonce,
        },
        id,
    )
    .await
    .map_err(|e| {
        error!(?e, "Integrity verification failed");
        ClientAuthError::IntegrityCheckFailed
    })?;

    if attestation != AttestationStatus::Attested {
        error!("Integrity attestation unavailable for client {id}");
        return Err(ClientAuthError::IntegrityCheckFailed);
    }

    Ok(())
}

/// Atomically increments the nonce and re-signs the integrity envelope.
/// Returns the new nonce, which is used as the challenge nonce.
async fn create_nonce(
    db: &db::DatabasePool,
    keyholder: &ActorRef<KeyHolder>,
    pubkey: &authn::PublicKey,
) -> Result<i32, ClientAuthError> {
    let pubkey_bytes = pubkey.to_bytes();
    let pubkey = pubkey.clone();

    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        ClientAuthError::DatabasePoolUnavailable
    })?;

    conn.exclusive_transaction(|conn| {
        let keyholder = keyholder.clone();
        let pubkey = pubkey.clone();
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
                    pubkey: pubkey.clone(),
                    nonce: new_nonce,
                },
                id,
            )
            .await
            .map_err(|e| {
                error!(?e, "Integrity sign failed after nonce update");
                ClientAuthError::DatabaseOperationFailed
            })?;

            Ok(new_nonce)
        })
    })
    .await
}

async fn approve_new_client(
    actors: &crate::actors::GlobalActors,
    profile: ClientProfile,
) -> Result<(), ClientAuthError> {
    let result = actors
        .flow_coordinator
        .ask(RequestClientApproval { client: profile })
        .await;

    match result {
        Ok(true) => Ok(()),
        Ok(false) => Err(ClientAuthError::ApproveError(ApproveError::Denied)),
        Err(SendError::HandlerError(e)) => {
            error!(error = ?e, "Approval upstream error");
            Err(ClientAuthError::ApproveError(ApproveError::Upstream(e)))
        }
        Err(e) => {
            error!(error = ?e, "Approval request to flow coordinator failed");
            Err(ClientAuthError::ApproveError(ApproveError::Internal))
        }
    }
}

async fn insert_client(
    db: &db::DatabasePool,
    keyholder: &ActorRef<KeyHolder>,
    pubkey: &authn::PublicKey,
    metadata: &ClientMetadata,
) -> Result<i32, ClientAuthError> {
    use crate::db::schema::client_metadata;
    let pubkey = pubkey.clone();
    let metadata = metadata.clone();

    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        ClientAuthError::DatabasePoolUnavailable
    })?;

    conn.exclusive_transaction(|conn| {
        let keyholder = keyholder.clone();
        let pubkey = pubkey.clone();
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
                    program_client::public_key.eq(pubkey.to_bytes()),
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
                    pubkey: pubkey.clone(),
                    nonce: NONCE_START,
                },
                client_id,
            )
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to sign integrity tag for new client key");
                ClientAuthError::DatabaseOperationFailed
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
) -> Result<(), ClientAuthError> {
    use crate::db::schema::{client_metadata, client_metadata_history};

    let now = SqliteTimestamp(Utc::now());

    let mut conn = db.get().await.map_err(|e| {
        error!(error = ?e, "Database pool error");
        ClientAuthError::DatabasePoolUnavailable
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
        ClientAuthError::DatabaseOperationFailed
    })
}

async fn challenge_client<T>(
    transport: &mut T,
    pubkey: authn::PublicKey,
    nonce: i32,
) -> Result<(), ClientAuthError>
where
    T: Bi<Inbound, Result<Outbound, ClientAuthError>> + ?Sized,
{
    transport
        .send(Ok(Outbound::AuthChallenge {
            pubkey: pubkey.clone(),
            nonce,
        }))
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to send auth challenge");
            ClientAuthError::Transport
        })?;

    let signature = expect_message(transport, |req: Inbound| match req {
        Inbound::AuthChallengeSolution { signature } => Some(signature),
        Inbound::AuthChallengeRequest { .. } => None,
    })
    .await
    .map_err(|e| {
        error!(error = ?e, "Failed to receive challenge solution");
        ClientAuthError::Transport
    })?;

    if !pubkey.verify(nonce, CLIENT_CONTEXT, &signature) {
        error!("Challenge solution verification failed");
        return Err(ClientAuthError::InvalidChallengeSolution);
    }

    Ok(())
}

pub async fn authenticate<T>(
    props: &mut ClientConnection,
    transport: &mut T,
) -> Result<i32, ClientAuthError>
where
    T: Bi<Inbound, Result<Outbound, ClientAuthError>> + Send + ?Sized,
{
    let Some(Inbound::AuthChallengeRequest { pubkey, metadata }) = transport.recv().await else {
        return Err(ClientAuthError::Transport);
    };

    let client_id = if let Some((id, _)) = get_current_nonce_and_id(&props.db, &pubkey).await? {
        verify_integrity(&props.db, &props.actors.key_holder, &pubkey).await?;
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
        insert_client(&props.db, &props.actors.key_holder, &pubkey, &metadata).await?
    };

    sync_client_metadata(&props.db, client_id, &metadata).await?;
    let challenge_nonce = create_nonce(&props.db, &props.actors.key_holder, &pubkey).await?;
    challenge_client(transport, pubkey, challenge_nonce).await?;

    transport
        .send(Ok(Outbound::AuthSuccess))
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to send auth success");
            ClientAuthError::Transport
        })?;

    Ok(client_id)
}
