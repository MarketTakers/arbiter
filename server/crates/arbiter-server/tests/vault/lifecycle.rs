use crate::common;
use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
use arbiter_server::{
    actors::{
        GlobalActors,
        vault::{Error, GetState, Vault, VaultState},
        vault_coordinator::{
            ContributeBootstrap, ContributeRecoveryBootstrap, ContributeRecoveryUnseal,
            ContributeUnseal, Error as CoordinatorError, StartBootstrap, VaultCoordinator,
        },
    },
    crypto::{KeyCell, encryption::v1::{Nonce, ROOT_KEY_TAG}},
    db::{self, models, schema},
};

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, insert_into, sql_query};
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

    // §3.6: the recovery operator's share only counts once a wake-up request has stood
    // uncancelled for the full dispute window, so back-date one here before unsealing.
    {
        let mut conn = db.get().await.unwrap();
        sql_query(format!(
            "INSERT INTO recovery_wakeup_request (requested_by, requested_at) \
             VALUES ({ordinary_id}, unixepoch('now') - 14*24*3600 - 1)"
        ))
        .execute(&mut conn)
        .await
        .unwrap();
    }

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

/// A committee of zero ordinary operators used to reach `shamir_threshold(0)` and panic,
/// taking the global coordinator down with it.
#[tokio::test]
#[test_log::test]
async fn empty_committee_is_rejected_without_panicking() {
    let db = db::create_test_pool().await;
    let bus = GlobalActors::spawn_message_bus();
    let vault_ref = Vault::spawn(Vault::new(db.clone(), bus).await.unwrap());
    let coordinator = VaultCoordinator::spawn(VaultCoordinator::new(db, vault_ref));

    let err = coordinator
        .ask(StartBootstrap {
            operator_id: 1,
            declared_count: 0,
            recovery_count: 1,
        })
        .await
        .unwrap_err();

    assert!(
        matches!(
            err,
            kameo::error::SendError::HandlerError(CoordinatorError::EmptyCommittee)
        ),
        "expected EmptyCommittee, got {err:?}"
    );

    // The actor must still be alive to serve the next caller.
    let err = coordinator
        .ask(StartBootstrap {
            operator_id: 1,
            declared_count: 0,
            recovery_count: 0,
        })
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        kameo::error::SendError::HandlerError(CoordinatorError::EmptyCommittee)
    ));
}

/// An approved-but-unfinished operator replacement deletes a share row. The unseal threshold
/// must still describe the split that is actually stored, not the surviving row count.
///
/// Four ordinary operators give a real 3-of-4 `vsss-rs` split (`shamir_threshold(4) == 3`).
/// Deleting one share row leaves 3 rows, and `shamir_threshold(3) == 2` -- a *different* number
/// from the recorded threshold. A recount-based unseal would therefore finalize one contribution
/// early, combine only 2 shares of a 3-of-4 split, and fail to reconstruct the seal key.
#[tokio::test]
#[test_log::test]
async fn unseal_threshold_survives_a_deleted_share_row() {
    let db = db::create_test_pool().await;
    let bus = GlobalActors::spawn_message_bus();
    let vault_ref = Vault::spawn(Vault::new(db.clone(), bus).await.unwrap());
    let coordinator = VaultCoordinator::spawn(VaultCoordinator::new(db.clone(), vault_ref));

    // Four ordinary operators: threshold is 3-of-4.
    let mut ids = Vec::new();
    for n in 1..=4u8 {
        let mut conn = db.get().await.unwrap();
        let id: i32 = insert_into(schema::operator_identity::table)
            .values(schema::operator_identity::public_key.eq(vec![n; 32]))
            .returning(schema::operator_identity::id)
            .get_result(&mut conn)
            .await
            .unwrap();
        ids.push(id);
    }

    coordinator
        .ask(StartBootstrap {
            operator_id: ids[0],
            declared_count: 4,
            recovery_count: 0,
        })
        .await
        .unwrap();
    for (n, id) in ids.iter().enumerate() {
        coordinator
            .ask(ContributeBootstrap {
                operator_id: *id,
                passphrase: SafeCell::new(format!("pass-{n}").into_bytes()),
            })
            .await
            .unwrap();
    }

    let stored: Option<i32> = {
        let mut conn = db.get().await.unwrap();
        schema::arbiter_settings::table
            .select(schema::arbiter_settings::shamir_threshold)
            .first(&mut conn)
            .await
            .unwrap()
    };
    assert_eq!(stored, Some(3), "bootstrap must record the split threshold");

    // Simulate the aborted replacement: one share row is gone, leaving 3 of the 4 shares.
    {
        let mut conn = db.get().await.unwrap();
        diesel::delete(schema::operator::table)
            .filter(schema::operator::id.eq(Some(ids[3])))
            .execute(&mut conn)
            .await
            .unwrap();
    }

    // Restart and unseal with the three surviving operators' original passphrases.
    drop(coordinator);
    let bus2 = GlobalActors::spawn_message_bus();
    let vault_ref2 = Vault::spawn(Vault::new(db.clone(), bus2).await.unwrap());
    let coordinator2 = VaultCoordinator::spawn(VaultCoordinator::new(db.clone(), vault_ref2.clone()));

    let done = coordinator2
        .ask(ContributeUnseal {
            operator_id: ids[0],
            passphrase: SafeCell::new(b"pass-0".to_vec()),
        })
        .await
        .unwrap();
    assert!(!done, "one share must not be enough for a 3-of-4 split");

    let done = coordinator2
        .ask(ContributeUnseal {
            operator_id: ids[1],
            passphrase: SafeCell::new(b"pass-1".to_vec()),
        })
        .await
        .unwrap();
    assert!(
        !done,
        "two shares must not be enough for a 3-of-4 split -- a recount would wrongly finalize here"
    );

    let done = coordinator2
        .ask(ContributeUnseal {
            operator_id: ids[2],
            passphrase: SafeCell::new(b"pass-2".to_vec()),
        })
        .await
        .unwrap();
    assert!(done, "three shares must reconstruct a 3-of-4 split");
    assert_eq!(
        vault_ref2.ask(GetState {}).await.unwrap(),
        VaultState::Unsealed
    );
}

/// §3.6: recovery operators are asleep by default. Without a wake-up whose 14-day dispute
/// window has elapsed, their share must not count towards an unseal.
#[tokio::test]
#[test_log::test]
async fn sleeping_recovery_operator_cannot_contribute_to_unseal() {
    let db = db::create_test_pool().await;
    let bus = GlobalActors::spawn_message_bus();
    let vault_ref = Vault::spawn(Vault::new(db.clone(), bus).await.unwrap());
    let coordinator = VaultCoordinator::spawn(VaultCoordinator::new(db.clone(), vault_ref.clone()));

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

    coordinator
        .ask(StartBootstrap {
            operator_id: ordinary_id,
            declared_count: 1,
            recovery_count: 1,
        })
        .await
        .unwrap();
    coordinator
        .ask(ContributeRecoveryBootstrap {
            recovery_operator_id: recovery_id,
            passphrase: SafeCell::new(b"recovery-pass".to_vec()),
        })
        .await
        .unwrap();
    coordinator
        .ask(ContributeBootstrap {
            operator_id: ordinary_id,
            passphrase: SafeCell::new(b"ordinary-pass".to_vec()),
        })
        .await
        .unwrap();

    // Restart so the vault comes up Sealed.
    drop(coordinator);
    drop(vault_ref);
    let bus2 = GlobalActors::spawn_message_bus();
    let vault_ref2 = Vault::spawn(Vault::new(db.clone(), bus2).await.unwrap());
    let coordinator2 = VaultCoordinator::spawn(VaultCoordinator::new(db.clone(), vault_ref2.clone()));

    let err = coordinator2
        .ask(ContributeRecoveryUnseal {
            recovery_operator_id: recovery_id,
            passphrase: SafeCell::new(b"recovery-pass".to_vec()),
        })
        .await
        .unwrap_err();

    assert!(
        matches!(
            err,
            kameo::error::SendError::HandlerError(CoordinatorError::RecoveryNotActive)
        ),
        "expected RecoveryNotActive, got {err:?}"
    );
    assert_eq!(
        vault_ref2.ask(GetState {}).await.unwrap(),
        VaultState::Sealed,
        "a sleeping recovery operator unsealed the vault"
    );
}
