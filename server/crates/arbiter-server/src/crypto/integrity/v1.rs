use crate::{
    actors::keyholder, crypto::integrity::hashing::Hashable
};
use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
use hmac::{Hmac, Mac as _};
use sha2::Sha256;

use diesel::{ExpressionMethods as _, QueryDsl, dsl::insert_into, sqlite::Sqlite};
use diesel_async::{AsyncConnection, RunQueryDsl};
use kameo::{actor::ActorRef, error::SendError};
use sha2::Digest as _;

pub mod hashing;

use crate::{
    actors::keyholder::{KeyHolder, SignIntegrity, VerifyIntegrity},
    db::{
        self,
        models::{IntegrityEnvelope, NewIntegrityEnvelope},
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
pub enum AttestationStatus {
    Attested,
    Unavailable,
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

pub trait IntoId {
    fn into_id(self) -> Vec<u8>;
}

impl IntoId for i32 {
    fn into_id(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl IntoId for &'_ [u8] {
    fn into_id(self) -> Vec<u8> {
        self.to_vec()
    }
}

pub async fn sign_entity<E: Integrable>(
    conn: &mut impl AsyncConnection<Backend = Sqlite>,
    keyholder: &ActorRef<KeyHolder>,
    entity: &E,
    entity_id: impl IntoId,
) -> Result<(), Error> {
    let payload_hash = payload_hash(&entity);

    let entity_id = entity_id.into_id();

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
            entity_id: entity_id,
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

    Ok(())
}

pub async fn verify_entity<E: Integrable>(
    conn: &mut impl AsyncConnection<Backend = Sqlite>,
    keyholder: &ActorRef<KeyHolder>,
    entity: &E,
    entity_id: impl IntoId,
) -> Result<AttestationStatus, Error> {
    let entity_id = entity_id.into_id();
    let envelope: IntegrityEnvelope = integrity_envelope::table
        .filter(integrity_envelope::entity_kind.eq(E::KIND))
        .filter(integrity_envelope::entity_id.eq(&entity_id))
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

    let payload_hash = payload_hash(&entity);
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

#[cfg(test)]
mod tests {
    use diesel::{ExpressionMethods as _, QueryDsl};
    use diesel_async::RunQueryDsl;
    use kameo::{actor::ActorRef, prelude::Spawn};
    use rand::seq::SliceRandom;
    use sha2::Digest;

    use proptest::prelude::*;

    use crate::{
        actors::keyholder::{Bootstrap, KeyHolder},
        db::{self, schema},
       
    };
    use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};

    use super::{Error, Integrable, sign_entity, verify_entity};
    use super::{hashing::Hashable, payload_hash};

    #[derive(Clone)]
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
        verify_entity(&mut conn, &keyholder, &entity, ENTITY_ID)
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

        let err = verify_entity(&mut conn, &keyholder, &entity, ENTITY_ID)
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

        let err = verify_entity(&mut conn, &keyholder, &tampered, ENTITY_ID)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::MacMismatch { .. }));
    }
}
