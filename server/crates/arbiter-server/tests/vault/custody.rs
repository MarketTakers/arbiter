//! End-to-end coverage for the Shamir custody ceremonies.

use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
use arbiter_server::{
    actors::{
        GlobalActors,
        vault::{Bootstrap, GetState, Seal, VaultState},
        vault_coordinator::{ContributeBootstrap, ContributeUnseal, StartBootstrap},
    },
    crypto::{KeyCell, shamir},
    db::{self, models::OperatorId, schema},
};

use diesel::{ExpressionMethods as _, QueryDsl};
use diesel_async::RunQueryDsl;

/// Register `count` operator identities so committee members satisfy the
/// foreign key from `operator` to `operator_identity`.
async fn register_operators(db: &db::DatabasePool, count: usize) -> Vec<OperatorId> {
    let mut conn = db.get().await.unwrap();
    let mut ids = Vec::with_capacity(count);
    for index in 0..count {
        let pubkey = vec![u8::try_from(index).unwrap(); 32];
        let id: OperatorId = diesel::insert_into(schema::operator_identity::table)
            .values((schema::operator_identity::public_key.eq(pubkey),))
            .returning(schema::operator_identity::id)
            .get_result(&mut conn)
            .await
            .unwrap();
        ids.push(id);
    }
    ids
}

async fn stored_share_count(db: &db::DatabasePool) -> i64 {
    let mut conn = db.get().await.unwrap();
    schema::operator::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap()
}

async fn stored_threshold(db: &db::DatabasePool) -> Option<i32> {
    let mut conn = db.get().await.unwrap();
    schema::arbiter_settings::table
        .select(schema::arbiter_settings::shamir_threshold)
        .first(&mut conn)
        .await
        .unwrap()
}

fn passphrase(seed: u8) -> SafeCell<Vec<u8>> {
    SafeCell::new(vec![seed; 16])
}

/// The happy path: three operators bootstrap, and any two of them reopen the
/// vault after it is sealed.
#[tokio::test]
#[test_log::test]
async fn committee_of_three_unseals_with_two_passphrases() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    let operators = register_operators(&db, 3).await;

    actors
        .vault_coordinator
        .ask(StartBootstrap {
            operator_id: operators[0],
            declared_count: 3,
        })
        .await
        .unwrap();

    for (index, operator_id) in operators.iter().enumerate() {
        let finished = actors
            .vault_coordinator
            .ask(ContributeBootstrap {
                operator_id: *operator_id,
                passphrase: passphrase(u8::try_from(index).unwrap()),
            })
            .await
            .unwrap();
        assert_eq!(
            finished,
            index == 2,
            "the ceremony finishes only on the last contribution"
        );
    }

    assert_eq!(
        actors.vault.ask(GetState {}).await.unwrap(),
        VaultState::Unsealed
    );
    assert_eq!(stored_share_count(&db).await, 3);
    assert_eq!(stored_threshold(&db).await, Some(2));

    actors.vault.ask(Seal {}).await.unwrap();

    let first = actors
        .vault_coordinator
        .ask(ContributeUnseal {
            operator_id: operators[0],
            passphrase: passphrase(0),
        })
        .await
        .unwrap();
    assert!(!first, "one of two shares must not unseal");

    let second = actors
        .vault_coordinator
        .ask(ContributeUnseal {
            operator_id: operators[2],
            passphrase: passphrase(2),
        })
        .await
        .unwrap();
    assert!(second, "the threshold contribution should unseal the vault");
    assert_eq!(
        actors.vault.ask(GetState {}).await.unwrap(),
        VaultState::Unsealed
    );
}

/// Shares that describe a seal key the vault never adopted would make the vault
/// permanently un-unsealable, so a refused bootstrap must leave the table empty.
#[tokio::test]
#[test_log::test]
async fn refused_bootstrap_stores_no_shares() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    let operators = register_operators(&db, 1).await;

    // Another path bootstraps the vault first; the ceremony now has nowhere to go.
    actors
        .vault
        .ask(Bootstrap {
            seal_key: KeyCell::from([4u8; 32]),
            custody: None,
        })
        .await
        .unwrap();

    actors
        .vault_coordinator
        .ask(StartBootstrap {
            operator_id: operators[0],
            declared_count: 1,
        })
        .await
        .unwrap();

    let error = actors
        .vault_coordinator
        .ask(ContributeBootstrap {
            operator_id: operators[0],
            passphrase: passphrase(1),
        })
        .await
        .expect_err("bootstrapping an already bootstrapped vault must fail");
    assert!(
        format!("{error:?}").contains("AlreadyBootstrapped"),
        "expected AlreadyBootstrapped, got {error:?}"
    );

    assert_eq!(
        stored_share_count(&db).await,
        0,
        "a refused bootstrap must not leave shares behind"
    );
    assert_eq!(
        stored_threshold(&db).await,
        None,
        "a refused bootstrap must not leave a threshold behind"
    );
}

#[tokio::test]
#[test_log::test]
async fn oversized_and_degenerate_committees_are_rejected() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    let operators = register_operators(&db, 1).await;

    for (count, expected) in [
        (0_usize, "EmptyCommittee"),
        (2, "UnsupportedCommittee"),
        (shamir::MAX_COMMITTEE_SIZE + 1, "CommitteeTooLarge"),
        (usize::MAX, "CommitteeTooLarge"),
    ] {
        let error = actors
            .vault_coordinator
            .ask(StartBootstrap {
                operator_id: operators[0],
                declared_count: count,
            })
            .await
            .expect_err("committee size must be rejected");
        assert!(
            format!("{error:?}").contains(expected),
            "expected {expected} for count {count}, got {error:?}"
        );
    }
}

/// A committee whose members never all show up would otherwise wedge the
/// coordinator until restart. Only the operator that declared it may reset it.
#[tokio::test]
#[test_log::test]
async fn only_the_declarer_may_restart_a_stalled_committee() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    let operators = register_operators(&db, 3).await;

    actors
        .vault_coordinator
        .ask(StartBootstrap {
            operator_id: operators[0],
            declared_count: 3,
        })
        .await
        .unwrap();
    actors
        .vault_coordinator
        .ask(ContributeBootstrap {
            operator_id: operators[0],
            passphrase: passphrase(0),
        })
        .await
        .unwrap();

    let error = actors
        .vault_coordinator
        .ask(StartBootstrap {
            operator_id: operators[1],
            declared_count: 3,
        })
        .await
        .expect_err("a bystander must not reset someone else's ceremony");
    assert!(
        format!("{error:?}").contains("AlreadyBootstrapping"),
        "expected AlreadyBootstrapping, got {error:?}"
    );

    actors
        .vault_coordinator
        .ask(StartBootstrap {
            operator_id: operators[0],
            declared_count: 1,
        })
        .await
        .expect("the declarer may restart the ceremony");

    let finished = actors
        .vault_coordinator
        .ask(ContributeBootstrap {
            operator_id: operators[0],
            passphrase: passphrase(0),
        })
        .await
        .unwrap();
    assert!(
        finished,
        "the restarted one-operator ceremony should complete"
    );
    assert_eq!(stored_share_count(&db).await, 1);
}

/// A wrong passphrase must not unseal, and the operator must be able to retry.
#[tokio::test]
#[test_log::test]
async fn wrong_passphrase_is_rejected_and_retryable() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    let operators = register_operators(&db, 1).await;

    actors
        .vault_coordinator
        .ask(StartBootstrap {
            operator_id: operators[0],
            declared_count: 1,
        })
        .await
        .unwrap();
    actors
        .vault_coordinator
        .ask(ContributeBootstrap {
            operator_id: operators[0],
            passphrase: passphrase(7),
        })
        .await
        .unwrap();
    actors.vault.ask(Seal {}).await.unwrap();

    let error = actors
        .vault_coordinator
        .ask(ContributeUnseal {
            operator_id: operators[0],
            passphrase: passphrase(8),
        })
        .await
        .expect_err("a wrong passphrase must not unseal");
    assert!(
        format!("{error:?}").contains("InvalidPassphrase"),
        "expected InvalidPassphrase, got {error:?}"
    );

    let unsealed = actors
        .vault_coordinator
        .ask(ContributeUnseal {
            operator_id: operators[0],
            passphrase: passphrase(7),
        })
        .await
        .expect("the operator may retry with the correct passphrase");
    assert!(unsealed, "the retry should unseal the vault");
}
