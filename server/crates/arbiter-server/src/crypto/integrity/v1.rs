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
use self::hashing::Hashable;

use crate::{
    actors::keyholder::{KeyHolder, SignIntegrity, VerifyIntegrity},
    db::{
        self,
        models::{IntegrityEnvelope as IntegrityEnvelopeRow, NewIntegrityEnvelope},
        schema::integrity_envelope,
    },
};

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

#[derive(Debug)]
pub struct Verified<T>(T);

impl<T> AsRef<T> for Verified<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Verified<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Verified<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub const CURRENT_PAYLOAD_VERSION: i32 = 1;
pub const INTEGRITY_SUBKEY_TAG: &[u8] = b"arbiter/db-integrity-key/v1";

pub type HmacSha256 = Hmac<Sha256>;

pub trait Integrable: Hashable {
    const KIND: &'static str;
    const VERSION: i32 = 1;
}

fn payload_hash(payload: &impl Hashable) -> [u8; 32] {
    let mut hasher = Sha256::new();
    payload.hash(&mut hasher);
    hasher.finalize().into()
}

fn push_len_prefixed(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    out.extend_from_slice(bytes);
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

pub async fn lookup_verified<E, C, F, Fut>(
    conn: &mut C,
    keyholder: &ActorRef<KeyHolder>,
    entity_id: impl Into<EntityId>,
    load: F,
) -> Result<Verified<E>, Error>
where
    C: AsyncConnection<Backend = Sqlite>,
    E: Integrable,
    F: FnOnce(&mut C) -> Fut,
    Fut: Future<Output = Result<E, db::DatabaseError>>,
{
    let entity = load(conn).await?;
    verify_entity(conn, keyholder, &entity, entity_id).await?;
    Ok(Verified(entity))
}

pub async fn lookup_verified_allow_unavailable<E, C, F, Fut>(
    conn: &mut C,
    keyholder: &ActorRef<KeyHolder>,
    entity_id: impl Into<EntityId>,
    load: F,
) -> Result<Verified<E>, Error>
where
    C: AsyncConnection<Backend = Sqlite>,
    E: Integrable+ 'static,
    F: FnOnce(&mut C) -> Fut,
    Fut: Future<Output = Result<E, db::DatabaseError>>,
{
    let entity = load(conn).await?;
    match check_entity_attestation(conn, keyholder, &entity, entity_id.into()).await? {
        // IMPORTANT: allow_unavailable mode must succeed with an unattested result when vault key
        // material is unavailable, otherwise integrity checks can be silently bypassed while sealed.
        AttestationStatus::Attested | AttestationStatus::Unavailable => Ok(Verified(entity)),
    }
}

pub async fn lookup_verified_from_query<E, Id, C, F>(
    conn: &mut C,
    keyholder: &ActorRef<KeyHolder>,
    load: F,
) -> Result<Verified<E>, Error>
where
    C: AsyncConnection<Backend = Sqlite> + Send,
    E: Integrable,
    Id: Into<EntityId>,
    F: for<'a> FnOnce(
        &'a mut C,
    ) -> Pin<
        Box<dyn Future<Output = Result<(Id, E), db::DatabaseError>> + Send + 'a>,
    >,
{
    let (entity_id, entity) = load(conn).await?;
    verify_entity(conn, keyholder, &entity, entity_id).await?;
    Ok(Verified(entity))
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

    Ok(Verified(as_entity_id))
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

pub async fn verify_entity<'a, E: Integrable>(
    conn: &mut impl AsyncConnection<Backend = Sqlite>,
    keyholder: &ActorRef<KeyHolder>,
    entity: &'a E,
    entity_id: impl Into<EntityId>,
) -> Result<Verified<&'a E>, Error> {
    match check_entity_attestation::<E>(conn, keyholder, entity, entity_id).await? {
        AttestationStatus::Attested => Ok(Verified(entity)),
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

#[cfg(test)]
mod tests {
    use diesel::{ExpressionMethods as _, QueryDsl};
    use diesel_async::RunQueryDsl;
    use kameo::{actor::ActorRef, prelude::Spawn};
    use sha2::Digest;

    use crate::{
        actors::keyholder::{Bootstrap, KeyHolder},
        db::{self, schema},
        safe_cell::{SafeCell, SafeCellHandle as _},
    };

    use super::hashing::Hashable;
    use super::{
        check_entity_attestation, AttestationStatus, Error, Integrable, lookup_verified,
        lookup_verified_allow_unavailable, lookup_verified_from_query, sign_entity, verify_entity,
    };

    #[derive(Clone, Debug)]
    struct DummyEntity {
        payload_version: i32,
        payload: Vec<u8>,
    }

    impl Hashable for DummyEntity {
        fn hash<H: Digest>(&self, hasher: &mut H) {
            self.payload_version.hash(hasher);
            self.payload.hash(hasher);
        }
    }
    impl Integrable for DummyEntity {
        const KIND: &'static str = "dummy_entity";
    }

    async fn bootstrapped_keyholder(db: &db::DatabasePool) -> ActorRef<KeyHolder> {
        let actor = KeyHolder::spawn(KeyHolder::new(db.clone()).await.unwrap());
        actor
            .ask(Bootstrap {
                seal_key_raw: SafeCell::new(b"integrity-test-seal-key".to_vec()),
            })
            .await
            .unwrap();
        actor
    }

    #[tokio::test]
    async fn sign_writes_envelope_and_verify_passes() {
        let db = db::create_test_pool().await;
        let keyholder = bootstrapped_keyholder(&db).await;
        let mut conn = db.get().await.unwrap();

        const ENTITY_ID: &[u8] = b"entity-id-7";

        let entity = DummyEntity {
            payload_version: 1,
            payload: b"payload-v1".to_vec(),
        };

        sign_entity(&mut conn, &keyholder, &entity, ENTITY_ID)
            .await
            .unwrap();

        let count: i64 = schema::integrity_envelope::table
            .filter(schema::integrity_envelope::entity_kind.eq("dummy_entity"))
            .filter(schema::integrity_envelope::entity_id.eq(ENTITY_ID))
            .count()
            .get_result(&mut conn)
            .await
            .unwrap();

        assert_eq!(count, 1, "envelope row must be created exactly once");
        let _ = check_entity_attestation(&mut conn, &keyholder, &entity, ENTITY_ID)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn tampered_mac_fails_verification() {
        let db = db::create_test_pool().await;
        let keyholder = bootstrapped_keyholder(&db).await;
        let mut conn = db.get().await.unwrap();

        const ENTITY_ID: &[u8] = b"entity-id-11";

        let entity = DummyEntity {
            payload_version: 1,
            payload: b"payload-v1".to_vec(),
        };

        sign_entity(&mut conn, &keyholder, &entity, ENTITY_ID)
            .await
            .unwrap();

        diesel::update(schema::integrity_envelope::table)
            .filter(schema::integrity_envelope::entity_kind.eq("dummy_entity"))
            .filter(schema::integrity_envelope::entity_id.eq(ENTITY_ID))
            .set(schema::integrity_envelope::mac.eq(vec![0u8; 32]))
            .execute(&mut conn)
            .await
            .unwrap();

        let err = check_entity_attestation(&mut conn, &keyholder, &entity, ENTITY_ID)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::MacMismatch { .. }));
    }

    #[tokio::test]
    async fn changed_payload_fails_verification() {
        let db = db::create_test_pool().await;
        let keyholder = bootstrapped_keyholder(&db).await;
        let mut conn = db.get().await.unwrap();

        const ENTITY_ID: &[u8] = b"entity-id-21";

        let entity = DummyEntity {
            payload_version: 1,
            payload: b"payload-v1".to_vec(),
        };

        sign_entity(&mut conn, &keyholder, &entity, ENTITY_ID)
            .await
            .unwrap();

        let tampered = DummyEntity {
            payload: b"payload-v1-but-tampered".to_vec(),
            ..entity
        };

        let err = check_entity_attestation(&mut conn, &keyholder, &tampered, ENTITY_ID)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::MacMismatch { .. }));
    }

    #[tokio::test]
    async fn allow_unavailable_lookup_passes_while_sealed() {
        let db = db::create_test_pool().await;
        let keyholder = bootstrapped_keyholder(&db).await;
        let mut conn = db.get().await.unwrap();

        const ENTITY_ID: &[u8] = b"entity-id-31";

        let entity = DummyEntity {
            payload_version: 1,
            payload: b"payload-v1".to_vec(),
        };

        sign_entity(&mut conn, &keyholder, &entity, ENTITY_ID)
            .await
            .unwrap();
        drop(keyholder);

        let sealed_keyholder = KeyHolder::spawn(KeyHolder::new(db.clone()).await.unwrap());
        let status = check_entity_attestation(&mut conn, &sealed_keyholder, &entity, ENTITY_ID)
            .await
            .unwrap();
        assert_eq!(status, AttestationStatus::Unavailable);

        #[expect(clippy::disallowed_methods, reason = "test only")]
        lookup_verified_allow_unavailable(&mut conn, &sealed_keyholder, ENTITY_ID, |_| async {
            Ok::<_, db::DatabaseError>(DummyEntity {
                payload_version: 1,
                payload: b"payload-v1".to_vec(),
            })
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn strict_verify_fails_closed_while_sealed() {
        let db = db::create_test_pool().await;
        let keyholder = bootstrapped_keyholder(&db).await;
        let mut conn = db.get().await.unwrap();

        const ENTITY_ID: &[u8] = b"entity-id-41";

        let entity = DummyEntity {
            payload_version: 1,
            payload: b"payload-v1".to_vec(),
        };

        sign_entity(&mut conn, &keyholder, &entity, ENTITY_ID)
            .await
            .unwrap();
        drop(keyholder);

        let sealed_keyholder = KeyHolder::spawn(KeyHolder::new(db.clone()).await.unwrap());

        let err = verify_entity(&mut conn, &sealed_keyholder, &entity, ENTITY_ID)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            Error::Keyholder(crate::actors::keyholder::Error::NotBootstrapped)
        ));

        let err = lookup_verified(&mut conn, &sealed_keyholder, ENTITY_ID, |_| async {
            Ok::<_, db::DatabaseError>(DummyEntity {
                payload_version: 1,
                payload: b"payload-v1".to_vec(),
            })
        })
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            Error::Keyholder(crate::actors::keyholder::Error::NotBootstrapped)
        ));
    }

    #[tokio::test]
    async fn lookup_verified_supports_loaded_aggregate() {
        let db = db::create_test_pool().await;
        let keyholder = bootstrapped_keyholder(&db).await;
        let mut conn = db.get().await.unwrap();

        const ENTITY_ID: i32 = 77;

        let entity = DummyEntity {
            payload_version: 1,
            payload: b"payload-v1".to_vec(),
        };

        sign_entity(&mut conn, &keyholder, &entity, ENTITY_ID)
            .await
            .unwrap();

        let verified = lookup_verified(&mut conn, &keyholder, ENTITY_ID, |_| async {
            Ok::<_, db::DatabaseError>(DummyEntity {
                payload_version: 1,
                payload: b"payload-v1".to_vec(),
            })
        })
        .await
        .unwrap();

        assert_eq!(verified.payload, b"payload-v1".to_vec());
    }

    #[tokio::test]
    async fn lookup_verified_allow_unavailable_works_while_sealed() {
        let db = db::create_test_pool().await;
        let keyholder = bootstrapped_keyholder(&db).await;
        let mut conn = db.get().await.unwrap();

        const ENTITY_ID: i32 = 78;

        let entity = DummyEntity {
            payload_version: 1,
            payload: b"payload-v1".to_vec(),
        };

        sign_entity(&mut conn, &keyholder, &entity, ENTITY_ID)
            .await
            .unwrap();
        drop(keyholder);

        let sealed_keyholder = KeyHolder::spawn(KeyHolder::new(db.clone()).await.unwrap());

        #[expect(clippy::disallowed_methods, reason = "test only")]
        lookup_verified_allow_unavailable(&mut conn, &sealed_keyholder, ENTITY_ID, |_| async {
            Ok::<_, db::DatabaseError>(DummyEntity {
                payload_version: 1,
                payload: b"payload-v1".to_vec(),
            })
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn extension_trait_lookup_verified_required_works() {
        let db = db::create_test_pool().await;
        let keyholder = bootstrapped_keyholder(&db).await;
        let mut conn = db.get().await.unwrap();

        const ENTITY_ID: i32 = 79;

        let entity = DummyEntity {
            payload_version: 1,
            payload: b"payload-v1".to_vec(),
        };

        sign_entity(&mut conn, &keyholder, &entity, ENTITY_ID)
            .await
            .unwrap();

        let verified = lookup_verified(&mut conn, &keyholder, ENTITY_ID, |_| {
            Box::pin(async {
                Ok::<_, db::DatabaseError>(DummyEntity {
                    payload_version: 1,
                    payload: b"payload-v1".to_vec(),
                })
            })
        })
        .await
        .unwrap();

        assert_eq!(verified.payload, b"payload-v1".to_vec());
    }

    #[tokio::test]
    async fn lookup_verified_from_query_helpers_work() {
        let db = db::create_test_pool().await;
        let keyholder = bootstrapped_keyholder(&db).await;
        let mut conn = db.get().await.unwrap();

        const ENTITY_ID: i32 = 80;

        let entity = DummyEntity {
            payload_version: 1,
            payload: b"payload-v1".to_vec(),
        };

        sign_entity(&mut conn, &keyholder, &entity, ENTITY_ID)
            .await
            .unwrap();

        let verified = lookup_verified_from_query(&mut conn, &keyholder, |_| {
            Box::pin(async {
                Ok::<_, db::DatabaseError>((
                    ENTITY_ID,
                    DummyEntity {
                        payload_version: 1,
                        payload: b"payload-v1".to_vec(),
                    },
                ))
            })
        })
        .await
        .unwrap();

        assert_eq!(verified.payload, b"payload-v1".to_vec());

        drop(keyholder);
        let sealed_keyholder = KeyHolder::spawn(KeyHolder::new(db.clone()).await.unwrap());

        let err = lookup_verified_from_query(&mut conn, &sealed_keyholder, |_| {
            Box::pin(async {
                Ok::<_, db::DatabaseError>((
                    ENTITY_ID,
                    DummyEntity {
                        payload_version: 1,
                        payload: b"payload-v1".to_vec(),
                    },
                ))
            })
        })
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            Error::Keyholder(crate::actors::keyholder::Error::NotBootstrapped)
        ));
    }
}
