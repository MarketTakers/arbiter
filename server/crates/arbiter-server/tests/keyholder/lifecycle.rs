use arbiter_server::{
    actors::keyholder::{Error, KeyHolder}, crypto::encryption::v1::{Nonce, ROOT_KEY_TAG}, db::{self, models, schema}, safe_cell::{SafeCell, SafeCellHandle as _}
};
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

use crate::common;

#[tokio::test]
#[test_log::test]
async fn test_bootstrap() {
    let db = db::create_test_pool().await;
    let mut actor = KeyHolder::new(db.clone()).await.unwrap();

    let seal_key = SafeCell::new(b"test-seal-key".to_vec());
    actor.bootstrap(seal_key).await.unwrap();

    let mut conn = db.get().await.unwrap();
    let row: models::RootKeyHistory = schema::root_key_history::table
        .select(models::RootKeyHistory::as_select())
        .first(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.schema_version, 1);
    assert_eq!(
        row.tag,
        ROOT_KEY_TAG
    );
    assert!(!row.ciphertext.is_empty());
    assert!(!row.salt.is_empty());
    assert_eq!(
        row.data_encryption_nonce,
        Nonce::default().to_vec()
    );
}

#[tokio::test]
#[test_log::test]
async fn test_bootstrap_rejects_double() {
    let db = db::create_test_pool().await;
    let mut actor = common::bootstrapped_keyholder(&db).await;

    let seal_key2 = SafeCell::new(b"test-seal-key".to_vec());
    let err = actor.bootstrap(seal_key2).await.unwrap_err();
    assert!(matches!(err, Error::AlreadyBootstrapped));
}

#[tokio::test]
#[test_log::test]
async fn test_create_new_before_bootstrap_fails() {
    let db = db::create_test_pool().await;
    let mut actor = KeyHolder::new(db).await.unwrap();

    let err = actor
        .create_new(SafeCell::new(b"data".to_vec()))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::NotBootstrapped));
}

#[tokio::test]
#[test_log::test]
async fn test_decrypt_before_bootstrap_fails() {
    let db = db::create_test_pool().await;
    let mut actor = KeyHolder::new(db).await.unwrap();

    let err = actor.decrypt(1).await.unwrap_err();
    assert!(matches!(err, Error::NotBootstrapped));
}

#[tokio::test]
#[test_log::test]
async fn test_new_restores_sealed_state() {
    let db = db::create_test_pool().await;
    let actor = common::bootstrapped_keyholder(&db).await;
    drop(actor);

    let mut actor2 = KeyHolder::new(db).await.unwrap();
    let err = actor2.decrypt(1).await.unwrap_err();
    assert!(matches!(err, Error::NotBootstrapped));
}

#[tokio::test]
#[test_log::test]
async fn test_unseal_correct_password() {
    let db = db::create_test_pool().await;
    let mut actor = common::bootstrapped_keyholder(&db).await;

    let plaintext = b"survive a restart";
    let aead_id = actor
        .create_new(SafeCell::new(plaintext.to_vec()))
        .await
        .unwrap();
    drop(actor);

    let mut actor = KeyHolder::new(db.clone()).await.unwrap();
    let seal_key = SafeCell::new(b"test-seal-key".to_vec());
    actor.try_unseal(seal_key).await.unwrap();

    let mut decrypted = actor.decrypt(aead_id).await.unwrap();
    assert_eq!(*decrypted.read(), plaintext);
}

#[tokio::test]
#[test_log::test]
async fn test_unseal_wrong_then_correct_password() {
    let db = db::create_test_pool().await;
    let mut actor = common::bootstrapped_keyholder(&db).await;

    let plaintext = b"important data";
    let aead_id = actor
        .create_new(SafeCell::new(plaintext.to_vec()))
        .await
        .unwrap();
    drop(actor);

    let mut actor = KeyHolder::new(db.clone()).await.unwrap();

    let bad_key = SafeCell::new(b"wrong-password".to_vec());
    let err = actor.try_unseal(bad_key).await.unwrap_err();
    assert!(matches!(err, Error::InvalidKey));

    let good_key = SafeCell::new(b"test-seal-key".to_vec());
    actor.try_unseal(good_key).await.unwrap();

    let mut decrypted = actor.decrypt(aead_id).await.unwrap();
    assert_eq!(*decrypted.read(), plaintext);
}
