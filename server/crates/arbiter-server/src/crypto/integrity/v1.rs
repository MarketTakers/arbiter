use crate::actors::keyholder;
use hmac::Hmac;
use sha2::Sha256;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;

use diesel::{ExpressionMethods as _, QueryDsl, dsl::insert_into, sqlite::Sqlite};
use diesel_async::{AsyncConnection, RunQueryDsl};
use kameo::{actor::ActorRef, error::SendError};
use sha2::Digest as _;

pub mod hashing;
pub mod verified;
use self::hashing::Hashable;

use crate::{
    actors::keyholder::{KeyHolder, SignIntegrity, VerifyIntegrity},
    db::{
        self,
        models::{IntegrityEnvelope as IntegrityEnvelopeRow, NewIntegrityEnvelope},
        schema::integrity_envelope,
    },
};

pub const CURRENT_PAYLOAD_VERSION: i32 = 1;
pub const INTEGRITY_SUBKEY_TAG: &[u8] = b"arbiter/db-integrity-key/v1";

pub type HmacSha256 = Hmac<Sha256>;
pub use self::verified::Verified;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] db::DatabaseError),

    #[error("KeyHolder error: {0}")]
    Keyholder(#[from] keyholder::Error),

    #[error("KeyHolder mailbox error")]
    KeyholderSend,

    #[error("Integrity envelope is missing for entity {entity_kind}")]
    MissingEnvelope { entity_kind: &'static str },

    #[error(
        "Integrity payload version mismatch for entity {entity_kind}: expected {expected}, found {found}"
    )]
    PayloadVersionMismatch {
        entity_kind: &'static str,
        expected: i32,
        found: i32,
    },

    #[error("Integrity MAC mismatch for entity {entity_kind}")]
    MacMismatch { entity_kind: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum AttestationStatus {
    Attested,
    Unavailable,
}

pub trait Integrable: Hashable {
    const KIND: &'static str;
    const VERSION: i32 = 1;
}

impl<T: Integrable> Integrable for &T {
    const KIND: &'static str = T::KIND;
    const VERSION: i32 = T::VERSION;
}

#[derive(Debug, Clone)]
pub struct EntityId(Vec<u8>);

impl Deref for EntityId {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i32> for EntityId {
    fn from(value: i32) -> Self {
        Self(value.to_be_bytes().to_vec())
    }
}

impl From<&'_ [u8]> for EntityId {
    fn from(bytes: &'_ [u8]) -> Self {
        Self(bytes.to_vec())
    }
}

pub async fn lookup_verified<E, Id, C, F, Fut>(
    conn: &mut C,
    keyholder: &ActorRef<KeyHolder>,
    entity_id: Id,
    load: F,
) -> Result<Verified<Entity<E, Id>>, Error>
where
    C: AsyncConnection<Backend = Sqlite>,
    E: Integrable,
    Id: Into<EntityId> + Clone,
    F: FnOnce(&mut C) -> Fut,
    Fut: Future<Output = Result<E, db::DatabaseError>>,
{
    let entity = load(conn).await?;
    verify_entity(conn, keyholder, entity, entity_id).await
}

pub async fn lookup_verified_from_query<E, Id, C, F>(
    conn: &mut C,
    keyholder: &ActorRef<KeyHolder>,
    load: F,
) -> Result<Verified<Entity<E, Id>>, Error>
where
    C: AsyncConnection<Backend = Sqlite> + Send,
    E: Integrable,
    Id: Into<EntityId> + Clone,
    F: for<'a> FnOnce(
        &'a mut C,
    ) -> Pin<
        Box<dyn Future<Output = Result<(Id, E), db::DatabaseError>> + Send + 'a>,
    >,
{
    let (entity_id, entity) = load(conn).await?;
    verify_entity(conn, keyholder, entity, entity_id).await
}

pub async fn sign_entity<E: Integrable, Id: Into<EntityId> + Clone>(
    conn: &mut impl AsyncConnection<Backend = Sqlite>,
    keyholder: &ActorRef<KeyHolder>,
    entity: &E,
    as_entity_id: Id,
) -> Result<Verified<Id>, Error> {
    let payload_hash = payload_hash(entity);

    let entity_id = as_entity_id.clone().into();

    let mac_input = build_mac_input(E::KIND, &entity_id, E::VERSION, &payload_hash);

    let (key_version, mac) = keyholder
        .ask(SignIntegrity { mac_input })
        .await
        .map_err(|err| match err {
            kameo::error::SendError::HandlerError(inner) => Error::Keyholder(inner),
            _ => Error::KeyholderSend,
        })?;

    insert_into(integrity_envelope::table)
        .values(NewIntegrityEnvelope {
            entity_kind: E::KIND.to_owned(),
            entity_id: entity_id.to_vec(),
            payload_version: E::VERSION,
            key_version,
            mac: mac.to_vec(),
        })
        .on_conflict((
            integrity_envelope::entity_id,
            integrity_envelope::entity_kind,
        ))
        .do_update()
        .set((
            integrity_envelope::payload_version.eq(E::VERSION),
            integrity_envelope::key_version.eq(key_version),
            integrity_envelope::mac.eq(mac),
        ))
        .execute(conn)
        .await
        .map_err(db::DatabaseError::from)?;

    Ok(Verified::new(as_entity_id))
}

pub async fn check_entity_attestation<E: Integrable>(
    conn: &mut impl AsyncConnection<Backend = Sqlite>,
    keyholder: &ActorRef<KeyHolder>,
    entity: &E,
    entity_id: impl Into<EntityId>,
) -> Result<AttestationStatus, Error> {
    let entity_id = entity_id.into();
    let envelope: IntegrityEnvelopeRow = integrity_envelope::table
        .filter(integrity_envelope::entity_kind.eq(E::KIND))
        .filter(integrity_envelope::entity_id.eq(&*entity_id))
        .first(conn)
        .await
        .map_err(|err| match err {
            diesel::result::Error::NotFound => Error::MissingEnvelope {
                entity_kind: E::KIND,
            },
            other => Error::Database(db::DatabaseError::from(other)),
        })?;

    if envelope.payload_version != E::VERSION {
        return Err(Error::PayloadVersionMismatch {
            entity_kind: E::KIND,
            expected: E::VERSION,
            found: envelope.payload_version,
        });
    }

    let payload_hash = payload_hash(entity);
    let mac_input = build_mac_input(E::KIND, &entity_id, envelope.payload_version, &payload_hash);

    let result = keyholder
        .ask(VerifyIntegrity {
            mac_input,
            expected_mac: envelope.mac,
            key_version: envelope.key_version,
        })
        .await;

    match result {
        Ok(true) => Ok(AttestationStatus::Attested),
        Ok(false) => Err(Error::MacMismatch {
            entity_kind: E::KIND,
        }),
        Err(SendError::HandlerError(keyholder::Error::NotBootstrapped)) => {
            Ok(AttestationStatus::Unavailable)
        }
        Err(_) => Err(Error::KeyholderSend),
    }
}

#[derive(Debug, Clone, crate::VerifiedFields!)]
#[repr(C)]
pub struct Entity<E, Id> {
    pub entity: E,
    pub entity_id: Id,
}

impl<E, Id> Deref for Entity<E, Id> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

pub async fn verify_entity<E: Integrable, Id: Into<EntityId> + Clone>(
    conn: &mut impl AsyncConnection<Backend = Sqlite>,
    keyholder: &ActorRef<KeyHolder>,
    entity: E,
    entity_id: Id,
) -> Result<Verified<Entity<E, Id>>, Error> {
    match check_entity_attestation(conn, keyholder, &entity, entity_id.clone()).await? {
        AttestationStatus::Attested => Ok(Verified::new(Entity { entity, entity_id })),
        AttestationStatus::Unavailable => Err(Error::Keyholder(keyholder::Error::NotBootstrapped)),
    }
}

pub async fn verify_entity_ref<'e, E: Integrable, Id: Into<EntityId> + Clone>(
    conn: &mut impl AsyncConnection<Backend = Sqlite>,
    keyholder: &ActorRef<KeyHolder>,
    entity: &'e E,
    entity_id: Id,
) -> Result<Verified<Entity<&'e E, Id>>, Error> {
    match check_entity_attestation(conn, keyholder, entity, entity_id.clone()).await? {
        AttestationStatus::Attested => Ok(Verified::new(Entity { entity, entity_id })),
        AttestationStatus::Unavailable => Err(Error::Keyholder(keyholder::Error::NotBootstrapped)),
    }
}

pub async fn delete_envelope<E: Integrable>(
    conn: &mut impl AsyncConnection<Backend = Sqlite>,
    entity_id: impl Into<EntityId>,
) -> Result<usize, Error> {
    let entity_id = entity_id.into();

    let affected = diesel::delete(
        integrity_envelope::table
            .filter(integrity_envelope::entity_kind.eq(E::KIND))
            .filter(integrity_envelope::entity_id.eq(&*entity_id)),
    )
    .execute(conn)
    .await
    .map_err(db::DatabaseError::from)?;

    Ok(affected)
}

fn payload_hash(payload: &impl Hashable) -> [u8; 32] {
    let mut hasher = Sha256::new();
    payload.hash(&mut hasher);
    hasher.finalize().into()
}

fn build_mac_input(
    entity_kind: &str,
    entity_id: &[u8],
    payload_version: i32,
    payload_hash: &[u8; 32],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + entity_kind.len() + entity_id.len() + 32);
    push_len_prefixed(&mut out, entity_kind.as_bytes());
    push_len_prefixed(&mut out, entity_id);
    out.extend_from_slice(&payload_version.to_be_bytes());
    out.extend_from_slice(payload_hash);
    out
}

fn push_len_prefixed(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    out.extend_from_slice(bytes);
}

#[cfg(test)]
mod tests;
