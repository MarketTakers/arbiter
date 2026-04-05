use diesel::{ExpressionMethods as _, QueryDsl, dsl::insert_into, sqlite::Sqlite};
use diesel_async::{AsyncConnection, RunQueryDsl};
use kameo::actor::ActorRef;
use sha2::{Digest as _, Sha256};

use crate::{
    actors::keyholder::{KeyHolder, SignIntegrity, VerifyIntegrity},
    db::{
        self,
        models::{IntegrityEnvelope, NewIntegrityEnvelope},
        schema::integrity_envelope,
    },
};

pub const CURRENT_PAYLOAD_VERSION: i32 = 1;

pub mod evm;

pub trait IntegrityEntity {
    fn entity_kind(&self) -> &'static str;
    fn entity_id_bytes(&self) -> Vec<u8>;
    fn payload_version(&self) -> i32;
    fn canonical_payload_bytes(&self) -> Vec<u8>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] db::DatabaseError),

    #[error("KeyHolder error: {0}")]
    Keyholder(#[from] crate::actors::keyholder::Error),

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

fn payload_hash(payload: &[u8]) -> [u8; 32] {
    Sha256::digest(payload).into()
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

pub async fn sign_entity(
    conn: &mut impl AsyncConnection<Backend = Sqlite>,
    keyholder: &ActorRef<KeyHolder>,
    entity: &impl IntegrityEntity,
) -> Result<(), Error> {
    let entity_kind = entity.entity_kind();
    let entity_id = entity.entity_id_bytes();
    let payload_version = entity.payload_version();
    let payload = entity.canonical_payload_bytes();
    let payload_hash = payload_hash(&payload);
    let mac_input = build_mac_input(entity_kind, &entity_id, payload_version, &payload_hash);

    let (key_version, mac) = keyholder
        .ask(SignIntegrity { mac_input })
        .await
        .map_err(|err| match err {
            kameo::error::SendError::HandlerError(inner) => Error::Keyholder(inner),
            _ => Error::KeyholderSend,
        })?;

    diesel::delete(integrity_envelope::table)
        .filter(integrity_envelope::entity_kind.eq(entity_kind))
        .filter(integrity_envelope::entity_id.eq(&entity_id))
        .execute(conn)
        .await
        .map_err(db::DatabaseError::from)?;

    insert_into(integrity_envelope::table)
        .values(NewIntegrityEnvelope {
            entity_kind: entity_kind.to_string(),
            entity_id,
            payload_version,
            key_version,
            mac,
        })
        .execute(conn)
        .await
        .map_err(db::DatabaseError::from)?;

    Ok(())
}

pub async fn verify_entity(
    conn: &mut impl AsyncConnection<Backend = Sqlite>,
    keyholder: &ActorRef<KeyHolder>,
    entity: &impl IntegrityEntity,
) -> Result<(), Error> {
    let entity_kind = entity.entity_kind();
    let entity_id = entity.entity_id_bytes();
    let expected_payload_version = entity.payload_version();

    let envelope: IntegrityEnvelope = integrity_envelope::table
        .filter(integrity_envelope::entity_kind.eq(entity_kind))
        .filter(integrity_envelope::entity_id.eq(&entity_id))
        .first(conn)
        .await
        .map_err(|err| match err {
            diesel::result::Error::NotFound => Error::MissingEnvelope { entity_kind },
            other => Error::Database(db::DatabaseError::from(other)),
        })?;

    if envelope.payload_version != expected_payload_version {
        return Err(Error::PayloadVersionMismatch {
            entity_kind,
            expected: expected_payload_version,
            found: envelope.payload_version,
        });
    }

    let payload = entity.canonical_payload_bytes();
    let payload_hash = payload_hash(&payload);
    let mac_input = build_mac_input(
        entity_kind,
        &entity_id,
        envelope.payload_version,
        &payload_hash,
    );

    let ok = keyholder
        .ask(VerifyIntegrity {
            mac_input,
            expected_mac: envelope.mac,
            key_version: envelope.key_version,
        })
        .await
        .map_err(|err| match err {
            kameo::error::SendError::HandlerError(inner) => Error::Keyholder(inner),
            _ => Error::KeyholderSend,
        })?;

    if !ok {
        return Err(Error::MacMismatch { entity_kind });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use diesel::{ExpressionMethods as _, QueryDsl};
    use diesel_async::RunQueryDsl;
    use kameo::{actor::ActorRef, prelude::Spawn};

    use crate::{
        actors::keyholder::{Bootstrap, KeyHolder},
        db::{self, schema},
        safe_cell::{SafeCell, SafeCellHandle as _},
    };

    use super::{Error, IntegrityEntity, sign_entity, verify_entity};

    #[derive(Clone)]
    struct DummyEntity {
        id: i32,
        payload_version: i32,
        payload: Vec<u8>,
    }

    impl IntegrityEntity for DummyEntity {
        fn entity_kind(&self) -> &'static str {
            "dummy_entity"
        }

        fn entity_id_bytes(&self) -> Vec<u8> {
            self.id.to_be_bytes().to_vec()
        }

        fn payload_version(&self) -> i32 {
            self.payload_version
        }

        fn canonical_payload_bytes(&self) -> Vec<u8> {
            self.payload.clone()
        }
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

        let entity = DummyEntity {
            id: 7,
            payload_version: 1,
            payload: b"payload-v1".to_vec(),
        };

        sign_entity(&mut conn, &keyholder, &entity).await.unwrap();

        let count: i64 = schema::integrity_envelope::table
            .filter(schema::integrity_envelope::entity_kind.eq("dummy_entity"))
            .filter(schema::integrity_envelope::entity_id.eq(entity.entity_id_bytes()))
            .count()
            .get_result(&mut conn)
            .await
            .unwrap();

        assert_eq!(count, 1, "envelope row must be created exactly once");
        verify_entity(&mut conn, &keyholder, &entity).await.unwrap();
    }

    #[tokio::test]
    async fn tampered_mac_fails_verification() {
        let db = db::create_test_pool().await;
        let keyholder = bootstrapped_keyholder(&db).await;
        let mut conn = db.get().await.unwrap();

        let entity = DummyEntity {
            id: 11,
            payload_version: 1,
            payload: b"payload-v1".to_vec(),
        };

        sign_entity(&mut conn, &keyholder, &entity).await.unwrap();

        diesel::update(schema::integrity_envelope::table)
            .filter(schema::integrity_envelope::entity_kind.eq("dummy_entity"))
            .filter(schema::integrity_envelope::entity_id.eq(entity.entity_id_bytes()))
            .set(schema::integrity_envelope::mac.eq(vec![0u8; 32]))
            .execute(&mut conn)
            .await
            .unwrap();

        let err = verify_entity(&mut conn, &keyholder, &entity)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::MacMismatch { .. }));
    }

    #[tokio::test]
    async fn changed_payload_fails_verification() {
        let db = db::create_test_pool().await;
        let keyholder = bootstrapped_keyholder(&db).await;
        let mut conn = db.get().await.unwrap();

        let entity = DummyEntity {
            id: 21,
            payload_version: 1,
            payload: b"payload-v1".to_vec(),
        };

        sign_entity(&mut conn, &keyholder, &entity).await.unwrap();

        let tampered = DummyEntity {
            payload: b"payload-v1-but-tampered".to_vec(),
            ..entity
        };

        let err = verify_entity(&mut conn, &keyholder, &tampered)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::MacMismatch { .. }));
    }
}
