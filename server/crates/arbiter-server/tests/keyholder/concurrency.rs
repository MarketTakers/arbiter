use std::collections::{HashMap, HashSet};

use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
use arbiter_server::{
    actors::keyholder::{CreateNew, Error, KeyHolder},
    db::{self, models, schema},
};

use diesel::{ExpressionMethods as _, QueryDsl, SelectableHelper, dsl::sql_query};
use diesel_async::RunQueryDsl;
use kameo::actor::{ActorRef, Spawn as _};
use tokio::task::JoinSet;

use crate::common;

async fn write_concurrently(
    actor: ActorRef<KeyHolder>,
    prefix: &'static str,
    count: usize,
) -> Vec<(i32, Vec<u8>)> {
    let mut set = JoinSet::new();
    for i in 0..count {
        let actor = actor.clone();
        set.spawn(async move {
            let plaintext = format!("{prefix}-{i}").into_bytes();
            let id = actor
                .ask(CreateNew {
                    plaintext: SafeCell::new(plaintext.clone()),
                })
                .await
                .unwrap();
            (id, plaintext)
        });
    }

    let mut out = Vec::with_capacity(count);
    while let Some(res) = set.join_next().await {
        out.push(res.unwrap());
    }
    out
}

#[tokio::test]
#[test_log::test]
async fn concurrent_create_new_no_duplicate_nonces_() {
    let db = db::create_test_pool().await;
    let actor = KeyHolder::spawn(common::bootstrapped_keyholder(&db).await);

    let writes = write_concurrently(actor, "nonce-unique", 32).await;
    assert_eq!(writes.len(), 32);

    let mut conn = db.get().await.unwrap();
    let rows: Vec<models::AeadEncrypted> = schema::aead_encrypted::table
        .select(models::AeadEncrypted::as_select())
        .load(&mut conn)
        .await
        .unwrap();
    assert_eq!(rows.len(), 32);

    let nonces: Vec<&Vec<u8>> = rows.iter().map(|r| &r.current_nonce).collect();
    let unique: HashSet<&Vec<u8>> = nonces.iter().copied().collect();
    assert_eq!(nonces.len(), unique.len(), "all nonces must be unique");
}

#[tokio::test]
#[test_log::test]
async fn concurrent_create_new_root_nonce_never_moves_backward() {
    let db = db::create_test_pool().await;
    let actor = KeyHolder::spawn(common::bootstrapped_keyholder(&db).await);

    write_concurrently(actor, "root-max", 24).await;

    let mut conn = db.get().await.unwrap();
    let rows: Vec<models::AeadEncrypted> = schema::aead_encrypted::table
        .select(models::AeadEncrypted::as_select())
        .load(&mut conn)
        .await
        .unwrap();
    let max_nonce = rows
        .iter()
        .map(|r| r.current_nonce.clone())
        .max()
        .expect("at least one row");

    let root_row: models::RootKeyHistory = schema::root_key_history::table
        .select(models::RootKeyHistory::as_select())
        .first(&mut conn)
        .await
        .unwrap();
    assert_eq!(root_row.data_encryption_nonce, max_nonce);
}

#[tokio::test]
#[test_log::test]
async fn insert_failure_does_not_create_partial_row() {
    let db = db::create_test_pool().await;
    let mut actor = common::bootstrapped_keyholder(&db).await;
    let root_key_history_id = common::root_key_history_id(&db).await;

    let mut conn = db.get().await.unwrap();
    let before_count: i64 = schema::aead_encrypted::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    let before_root_nonce: Vec<u8> = schema::root_key_history::table
        .filter(schema::root_key_history::id.eq(root_key_history_id))
        .select(schema::root_key_history::data_encryption_nonce)
        .first(&mut conn)
        .await
        .unwrap();

    sql_query(
        "CREATE TRIGGER fail_aead_insert BEFORE INSERT ON aead_encrypted BEGIN SELECT RAISE(ABORT, 'forced test failure'); END;",
    )
    .execute(&mut conn)
    .await
    .unwrap();
    drop(conn);

    let err = actor
        .create_new(SafeCell::new(b"should fail".to_vec()))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::DatabaseTransaction(_)));

    let mut conn = db.get().await.unwrap();
    sql_query("DROP TRIGGER fail_aead_insert;")
        .execute(&mut conn)
        .await
        .unwrap();

    let after_count: i64 = schema::aead_encrypted::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(
        before_count, after_count,
        "failed insert must not create row"
    );

    let after_root_nonce: Vec<u8> = schema::root_key_history::table
        .filter(schema::root_key_history::id.eq(root_key_history_id))
        .select(schema::root_key_history::data_encryption_nonce)
        .first(&mut conn)
        .await
        .unwrap();
    assert!(
        after_root_nonce > before_root_nonce,
        "current behavior allows nonce gap on failed insert"
    );
}

#[tokio::test]
#[test_log::test]
async fn decrypt_roundtrip_after_high_concurrency() {
    let db = db::create_test_pool().await;
    let actor = KeyHolder::spawn(common::bootstrapped_keyholder(&db).await);

    let writes = write_concurrently(actor, "roundtrip", 40).await;
    let expected: HashMap<i32, Vec<u8>> = writes.into_iter().collect();

    let mut decryptor = KeyHolder::new(db.clone()).await.unwrap();
    decryptor
        .try_unseal(SafeCell::new(b"test-seal-key".to_vec()))
        .await
        .unwrap();

    for (id, plaintext) in expected {
        let mut decrypted = decryptor.decrypt(id).await.unwrap();
        assert_eq!(*decrypted.read(), plaintext);
    }
}
