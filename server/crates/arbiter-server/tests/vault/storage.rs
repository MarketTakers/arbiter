use crate::common;
use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
use arbiter_server::{
    actors::vault::Error,
    crypto::encryption::v1::Nonce,
    db::{self, models, schema},
};

use diesel::{ExpressionMethods as _, QueryDsl, SelectableHelper, dsl::update};
use diesel_async::RunQueryDsl;
use std::collections::HashSet;

#[tokio::test]
#[test_log::test]
async fn create_decrypt_roundtrip() {
    let db = db::create_test_pool().await;
    let mut actor = common::bootstrapped_vault(&db).await;

    let plaintext = b"hello arbiter";
    let aead_id = actor
        .create_new(SafeCell::new(plaintext.to_vec()))
        .await
        .unwrap();

    let mut decrypted = actor.decrypt(aead_id).await.unwrap();
    assert_eq!(*decrypted.read(), plaintext);
}

#[tokio::test]
#[test_log::test]
async fn decrypt_nonexistent_returns_not_found() {
    let db = db::create_test_pool().await;
    let mut actor = common::bootstrapped_vault(&db).await;

    let err = actor.decrypt(9999).await.unwrap_err();
    assert!(matches!(err, Error::NotFound));
}

#[tokio::test]
#[test_log::test]
async fn ciphertext_differs_across_entries() {
    let db = db::create_test_pool().await;
    let mut actor = common::bootstrapped_vault(&db).await;

    let plaintext = b"same content";
    let id1 = actor
        .create_new(SafeCell::new(plaintext.to_vec()))
        .await
        .unwrap();
    let id2 = actor
        .create_new(SafeCell::new(plaintext.to_vec()))
        .await
        .unwrap();

    let mut conn = db.get().await.unwrap();
    let row1: models::AeadEncrypted = schema::aead_encrypted::table
        .filter(schema::aead_encrypted::id.eq(id1))
        .select(models::AeadEncrypted::as_select())
        .first(&mut conn)
        .await
        .unwrap();
    let row2: models::AeadEncrypted = schema::aead_encrypted::table
        .filter(schema::aead_encrypted::id.eq(id2))
        .select(models::AeadEncrypted::as_select())
        .first(&mut conn)
        .await
        .unwrap();

    assert_ne!(row1.ciphertext, row2.ciphertext);

    let mut d1 = actor.decrypt(id1).await.unwrap();
    let mut d2 = actor.decrypt(id2).await.unwrap();
    assert_eq!(*d1.read(), plaintext);
    assert_eq!(*d2.read(), plaintext);
}

#[tokio::test]
#[test_log::test]
async fn nonce_never_reused() {
    let db = db::create_test_pool().await;
    let mut actor = common::bootstrapped_vault(&db).await;

    let n = 5;
    for i in 0..n {
        actor
            .create_new(SafeCell::new(format!("secret {i}").into_bytes()))
            .await
            .unwrap();
    }

    let mut conn = db.get().await.unwrap();
    let rows: Vec<models::AeadEncrypted> = schema::aead_encrypted::table
        .select(models::AeadEncrypted::as_select())
        .load(&mut conn)
        .await
        .unwrap();

    assert_eq!(rows.len(), n);

    let nonces: Vec<&Vec<u8>> = rows.iter().map(|r| &r.current_nonce).collect();
    let unique: HashSet<&Vec<u8>> = nonces.iter().copied().collect();
    assert_eq!(nonces.len(), unique.len(), "all nonces must be unique");

    for (i, row) in rows.iter().enumerate() {
        let mut expected = Nonce::default();
        for _ in 0..=i {
            expected.increment();
        }
        assert_eq!(row.current_nonce, expected.to_vec(), "nonce {i} mismatch");
    }

    let root_row: models::RootKeyHistory = schema::root_key_history::table
        .select(models::RootKeyHistory::as_select())
        .first(&mut conn)
        .await
        .unwrap();
    let last_nonce = &rows.last().unwrap().current_nonce;
    assert_eq!(&root_row.data_encryption_nonce, last_nonce);
}

#[tokio::test]
#[test_log::test]
async fn broken_db_nonce_format_fails_closed() {
    let db = db::create_test_pool().await;
    let mut actor = common::bootstrapped_vault(&db).await;
    let root_key_history_id = common::root_key_history_id(&db).await;

    let mut conn = db.get().await.unwrap();
    update(
        schema::root_key_history::table
            .filter(schema::root_key_history::id.eq(root_key_history_id)),
    )
    .set(schema::root_key_history::data_encryption_nonce.eq(vec![1, 2, 3]))
    .execute(&mut conn)
    .await
    .unwrap();
    drop(conn);

    let err = actor
        .create_new(SafeCell::new(b"must fail".to_vec()))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::BrokenDatabase));

    let db = db::create_test_pool().await;
    let mut actor = common::bootstrapped_vault(&db).await;
    let id = actor
        .create_new(SafeCell::new(b"decrypt target".to_vec()))
        .await
        .unwrap();
    let mut conn = db.get().await.unwrap();
    update(schema::aead_encrypted::table.filter(schema::aead_encrypted::id.eq(id)))
        .set(schema::aead_encrypted::current_nonce.eq(vec![7, 8]))
        .execute(&mut conn)
        .await
        .unwrap();
    drop(conn);

    let err = actor.decrypt(id).await.unwrap_err();
    assert!(matches!(err, Error::BrokenDatabase));
}
