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
    Error, Integrable, check_entity_attestation, lookup_verified, lookup_verified_from_query,
    sign_entity, verify_entity,
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
        .unwrap()
        .drop_verification_provenance();

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
        .unwrap()
        .drop_verification_provenance();

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
        .unwrap()
        .drop_verification_provenance();

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
        .unwrap()
        .drop_verification_provenance();
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
        .unwrap()
        .drop_verification_provenance();

    let verified = lookup_verified(&mut conn, &keyholder, ENTITY_ID, |_| async {
        Ok::<_, db::DatabaseError>(DummyEntity {
            payload_version: 1,
            payload: b"payload-v1".to_vec(),
        })
    })
    .await
    .unwrap();

    assert_eq!(verified.entity.payload, b"payload-v1".to_vec());
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
        .unwrap()
        .drop_verification_provenance();

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

    assert_eq!(verified.entity.payload, b"payload-v1".to_vec());
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
        .unwrap()
        .drop_verification_provenance();

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

    assert_eq!(verified.entity.payload, b"payload-v1".to_vec());

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
