use crate::common;
use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
use arbiter_server::{
    actors::{
        GlobalActors,
        vault::{Error, GetState, Vault, VaultState},
        vault_coordinator::{
            ContributeBootstrap, ContributeRecoveryBootstrap, ContributeRecoveryUnseal,
            Error as CoordinatorError, StartBootstrap, VaultCoordinator,
        },
    },
    crypto::{KeyCell, encryption::v1::{Nonce, ROOT_KEY_TAG}},
    db::{self, models, schema},
};

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, insert_into};
use diesel_async::RunQueryDsl;
use kameo::actor::Spawn as _;

#[tokio::test]
#[test_log::test]
async fn test_bootstrap() {
    let db = db::create_test_pool().await;
    let mut actor = Vault::new(db.clone(), GlobalActors::spawn_message_bus())
        .await
        .unwrap();

    let seal_key = KeyCell::from([0u8; 32]);
    actor.bootstrap(seal_key).await.unwrap();

    let mut conn = db.get().await.unwrap();
    let row: models::RootKeyHistory = schema::root_key_history::table
        .select(models::RootKeyHistory::as_select())
        .first(&mut conn)
        .await
        .unwrap();

    assert_eq!(row.schema_version, 1);
    assert_eq!(row.tag, ROOT_KEY_TAG);
    assert!(!row.ciphertext.is_empty());
    assert!(!row.salt.is_empty());
    assert_eq!(row.data_encryption_nonce, Nonce::default().to_vec());
}

#[tokio::test]
#[test_log::test]
async fn test_bootstrap_rejects_double() {
    let db = db::create_test_pool().await;
    let mut actor = common::bootstrapped_vault(&db).await;

    let seal_key2 = KeyCell::from([0u8; 32]);
    let err = actor.bootstrap(seal_key2).await.unwrap_err();
    assert!(matches!(err, Error::AlreadyBootstrapped));
}

#[tokio::test]
#[test_log::test]
async fn test_create_new_before_bootstrap_fails() {
    let db = db::create_test_pool().await;
    let mut actor = Vault::new(db, GlobalActors::spawn_message_bus())
        .await
        .unwrap();

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
    let mut actor = Vault::new(db, GlobalActors::spawn_message_bus())
        .await
        .unwrap();

    let err = actor.decrypt(1).await.unwrap_err();
    assert!(matches!(err, Error::NotBootstrapped));
}

#[tokio::test]
#[test_log::test]
async fn test_new_restores_sealed_state() {
    let db = db::create_test_pool().await;
    let actor = common::bootstrapped_vault(&db).await;
    drop(actor);

    let mut actor2 = Vault::new(db, GlobalActors::spawn_message_bus())
        .await
        .unwrap();
    let err = actor2.decrypt(1).await.unwrap_err();
    assert!(matches!(err, Error::Sealed));
}

#[tokio::test]
#[test_log::test]
async fn test_unseal_correct_password() {
    let db = db::create_test_pool().await;
    let mut actor = common::bootstrapped_vault(&db).await;

    let plaintext = b"survive a restart";
    let aead_id = actor
        .create_new(SafeCell::new(plaintext.to_vec()))
        .await
        .unwrap();
    drop(actor);

    let mut actor = Vault::new(db.clone(), GlobalActors::spawn_message_bus())
        .await
        .unwrap();
    let seal_key = KeyCell::from([0u8; 32]);
    actor.try_unseal(seal_key).await.unwrap();

    let mut decrypted = actor.decrypt(aead_id).await.unwrap();
    assert_eq!(*decrypted.read(), plaintext);
}

#[tokio::test]
#[test_log::test]
async fn test_unseal_wrong_then_correct_password() {
    let db = db::create_test_pool().await;
    let mut actor = common::bootstrapped_vault(&db).await;

    let plaintext = b"important data";
    let aead_id = actor
        .create_new(SafeCell::new(plaintext.to_vec()))
        .await
        .unwrap();
    drop(actor);

    let mut actor = Vault::new(db.clone(), GlobalActors::spawn_message_bus())
        .await
        .unwrap();

    let bad_key = KeyCell::from([1u8; 32]);
    let err = actor.try_unseal(bad_key).await.unwrap_err();
    assert!(matches!(err, Error::InvalidKey));

    let good_key = KeyCell::from([0u8; 32]);
    actor.try_unseal(good_key).await.unwrap();

    let mut decrypted = actor.decrypt(aead_id).await.unwrap();
    assert_eq!(*decrypted.read(), plaintext);
}

#[tokio::test]
#[test_log::test]
async fn two_operator_vault_requires_recovery_share() {
    let db = db::create_test_pool().await;
    let bus = GlobalActors::spawn_message_bus();
    let vault_ref = Vault::spawn(Vault::new(db.clone(), bus).await.unwrap());
    let coordinator = VaultCoordinator::spawn(VaultCoordinator::new(db, vault_ref));

    let err = coordinator
        .ask(StartBootstrap {
            operator_id: 1,
            declared_count: 2,
            recovery_count: 0,
        })
        .await
        .unwrap_err();

    assert!(
        matches!(
            err,
            kameo::error::SendError::HandlerError(CoordinatorError::TwoOperatorsRequireRecovery)
        ),
        "expected TwoOperatorsRequireRecovery, got {err:?}"
    );
}

/// §3.4: Bootstrap with 1 ordinary + 1 recovery operator produces a valid 1-of-2 Shamir split.
/// Both ordinary and recovery shares are stored; the vault can be unsealed with either one.
#[tokio::test]
#[test_log::test]
async fn recovery_share_stored_and_used_for_unseal() {
    let db = db::create_test_pool().await;
    let bus = GlobalActors::spawn_message_bus();
    let vault_ref = Vault::spawn(Vault::new(db.clone(), bus).await.unwrap());
    let coordinator = VaultCoordinator::spawn(VaultCoordinator::new(db.clone(), vault_ref.clone()));

    // Register one ordinary operator and one recovery operator in the DB
    let ordinary_id: i32 = {
        let mut conn = db.get().await.unwrap();
        insert_into(schema::operator_identity::table)
            .values(schema::operator_identity::public_key.eq(vec![1u8; 32]))
            .returning(schema::operator_identity::id)
            .get_result(&mut conn)
            .await
            .unwrap()
    };
    let recovery_id: i32 = {
        let mut conn = db.get().await.unwrap();
        insert_into(schema::recovery_operator_identity::table)
            .values(schema::recovery_operator_identity::public_key.eq(vec![2u8; 32]))
            .returning(schema::recovery_operator_identity::id)
            .get_result(&mut conn)
            .await
            .unwrap()
    };

    // Declare committee: 1 ordinary + 1 recovery
    coordinator
        .ask(StartBootstrap {
            operator_id: ordinary_id,
            declared_count: 1,
            recovery_count: 1,
        })
        .await
        .unwrap();

    // Recovery operator contributes first — bootstrap should not finalize yet
    let done = coordinator
        .ask(ContributeRecoveryBootstrap {
            recovery_operator_id: recovery_id,
            passphrase: SafeCell::new(b"recovery-pass".to_vec()),
        })
        .await
        .unwrap();
    assert!(!done, "should not finalize with only recovery passphrase");

    // Ordinary operator contributes — now bootstrap finalizes
    let done = coordinator
        .ask(ContributeBootstrap {
            operator_id: ordinary_id,
            passphrase: SafeCell::new(b"ordinary-pass".to_vec()),
        })
        .await
        .unwrap();
    assert!(done, "should finalize once all contributors are in");

    // After bootstrap, vault is Unsealed (seal key still in memory).
    let state = vault_ref.ask(GetState {}).await.unwrap();
    assert_eq!(state, VaultState::Unsealed);

    // Verify recovery_operator row was created
    let recovery_share_count: i64 = {
        let mut conn = db.get().await.unwrap();
        schema::recovery_operator::table
            .count()
            .get_result(&mut conn)
            .await
            .unwrap()
    };
    assert_eq!(recovery_share_count, 1);

    // Simulate restart: drop vault and coordinator, create fresh vault (comes up Sealed).
    drop(coordinator);
    drop(vault_ref);
    let bus2 = GlobalActors::spawn_message_bus();
    let vault_ref2 = Vault::spawn(Vault::new(db.clone(), bus2).await.unwrap());
    let state = vault_ref2.ask(GetState {}).await.unwrap();
    assert_eq!(state, VaultState::Sealed);

    // §3.5: Unseal using ONLY the recovery operator share (threshold = shamir_threshold(1) = 1).
    let coordinator2 = VaultCoordinator::spawn(VaultCoordinator::new(db.clone(), vault_ref2.clone()));
    let done = coordinator2
        .ask(ContributeRecoveryUnseal {
            recovery_operator_id: recovery_id,
            passphrase: SafeCell::new(b"recovery-pass".to_vec()),
        })
        .await
        .unwrap();
    assert!(done, "recovery share alone should satisfy threshold");

    let state = vault_ref2.ask(GetState {}).await.unwrap();
    assert_eq!(state, VaultState::Unsealed);
}
