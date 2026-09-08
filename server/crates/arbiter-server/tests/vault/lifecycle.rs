use crate::common;
use arbiter_crypto::{
    authn,
    safecell::{SafeCell, SafeCellHandle as _},
};
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
    peers::operator::{
        AuthenticatedOperator, Credentials, RecoveryCredentials,
        vault_gate::{
            Error as VaultGateError, HandleBootstrapEncryptedKey,
            HandleContributeBootstrapPassphrase, HandleContributeRecoveryBootstrapPassphrase,
            HandleContributeRecoveryUnsealPassphrase, HandleContributeUnsealPassphrase,
            HandleDeclareCommittee, HandleHandshake, VaultGate,
        },
    },
};

use chacha20poly1305::{AeadInPlace, XChaCha20Poly1305, XNonce, aead::KeyInit};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, insert_into, sql_query};
use diesel_async::RunQueryDsl;
use kameo::actor::Spawn as _;
use tokio::sync::oneshot;
use x25519_dalek::{EphemeralSecret, PublicKey};

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

type PromotionRx = oneshot::Receiver<Result<(), VaultGateError>>;

/// One `VaultGate` per authenticated role against a shared `GlobalActors`, which is what
/// `peers::operator::start` builds for two connected peers.
struct RoleGates {
    ordinary: kameo::actor::ActorRef<VaultGate>,
    recovery: kameo::actor::ActorRef<VaultGate>,
    ordinary_id: i32,
    recovery_id: i32,
    /// Held only so the gates' promotion channels stay open for the fixture's lifetime.
    _promotions: (PromotionRx, PromotionRx),
}

/// Registers one ordinary and one recovery identity, then spawns a gate for each.
async fn spawn_role_gates(db: &db::DatabasePool, actors: &GlobalActors) -> RoleGates {
    let ordinary_pubkey = authn::SigningKey::generate().public_key();
    let recovery_pubkey = authn::SigningKey::generate().public_key();

    let ordinary_id: i32 = {
        let mut conn = db.get().await.unwrap();
        insert_into(schema::operator_identity::table)
            .values(schema::operator_identity::public_key.eq(ordinary_pubkey.to_bytes()))
            .returning(schema::operator_identity::id)
            .get_result(&mut conn)
            .await
            .unwrap()
    };
    let recovery_id: i32 = {
        let mut conn = db.get().await.unwrap();
        insert_into(schema::recovery_operator_identity::table)
            .values(schema::recovery_operator_identity::public_key.eq(recovery_pubkey.to_bytes()))
            .returning(schema::recovery_operator_identity::id)
            .get_result(&mut conn)
            .await
            .unwrap()
    };

    let (ordinary_promotion_tx, ordinary_promotion_rx) = oneshot::channel();
    let ordinary = VaultGate::spawn(VaultGate::new(
        AuthenticatedOperator::Ordinary(Credentials {
            id: ordinary_id,
            pubkey: ordinary_pubkey,
        }),
        actors.clone(),
        db.clone(),
        ordinary_promotion_tx,
    ));

    let (recovery_promotion_tx, recovery_promotion_rx) = oneshot::channel();
    let recovery = VaultGate::spawn(VaultGate::new(
        AuthenticatedOperator::Recovery(RecoveryCredentials {
            id: recovery_id,
            pubkey: recovery_pubkey,
        }),
        actors.clone(),
        db.clone(),
        recovery_promotion_tx,
    ));

    RoleGates {
        ordinary,
        recovery,
        ordinary_id,
        recovery_id,
        _promotions: (ordinary_promotion_rx, recovery_promotion_rx),
    }
}

/// Runs the gate's X25519 handshake and encrypts `seal_key` to the shared secret, producing the
/// message a peer would send to bootstrap the vault. Mirrors `tests/operator/unseal.rs`'s
/// `client_dh_encrypt`, which does the same for the unseal side.
async fn bootstrap_key_for(
    gate: &kameo::actor::ActorRef<VaultGate>,
    seal_key: &[u8; 32],
) -> HandleBootstrapEncryptedKey {
    let client_secret = EphemeralSecret::random();
    let client_public = PublicKey::from(&client_secret);

    let response = gate
        .ask(HandleHandshake {
            client_pubkey: client_public,
        })
        .await
        .unwrap();

    let shared_secret = client_secret.diffie_hellman(&response.server_pubkey);
    let cipher = XChaCha20Poly1305::new(shared_secret.as_bytes().into());
    let nonce = XNonce::from([0u8; 24]);
    let associated_data = b"bootstrap";
    let mut ciphertext = seal_key.to_vec();
    cipher
        .encrypt_in_place(&nonce, associated_data, &mut ciphertext)
        .unwrap();

    HandleBootstrapEncryptedKey {
        nonce: nonce.to_vec(),
        ciphertext,
        associated_data: associated_data.to_vec(),
    }
}

/// Asserts a gate turned a request down on the peer's role rather than on anything else --
/// notably not on coordinator state, which is what an unguarded handler would have reported.
#[track_caller]
fn assert_role_refused<T: std::fmt::Debug, M>(
    what: &str,
    result: Result<T, kameo::error::SendError<M, VaultGateError>>,
) {
    match result {
        Err(kameo::error::SendError::HandlerError(VaultGateError::RoleNotPermitted)) => {}
        other => panic!("{what}: expected RoleNotPermitted, got {other:?}"),
    }
}

/// §3.5: which committee seat a passphrase fills is decided by the handshake, not by the
/// request, so neither role can spend the other's slot.
///
/// Neither request carries an operator id, so the ordinary peer has nothing left to forge; the
/// point of running the bootstrap to completion afterwards is that its refusal left the
/// recovery seat empty rather than filling it under a chosen id.
#[tokio::test]
#[test_log::test]
async fn ordinary_operator_cannot_contribute_a_recovery_share() {
    let db = db::create_test_pool().await;
    let actors = common::spawn_actors(db.clone()).await;
    let gates = spawn_role_gates(&db, &actors).await;
    assert_eq!(
        gates.ordinary_id, gates.recovery_id,
        "the two ids must collide for the attestation check at the end to mean anything"
    );

    gates
        .ordinary
        .ask(HandleDeclareCommittee {
            count: 1,
            recovery_count: 1,
        })
        .await
        .unwrap();

    assert_role_refused(
        "an ordinary operator contributed a recovery share",
        gates
            .ordinary
            .ask(HandleContributeRecoveryBootstrapPassphrase {
                passphrase: b"forged-recovery-pass".to_vec(),
            })
            .await,
    );

    assert_role_refused(
        "a recovery operator contributed an ordinary share",
        gates
            .recovery
            .ask(HandleContributeBootstrapPassphrase {
                passphrase: b"forged-ordinary-pass".to_vec(),
            })
            .await,
    );

    // The recovery seat is still empty: had the forged contribution landed, this one would come
    // back as a duplicate instead of being accepted.
    let done = gates
        .recovery
        .ask(HandleContributeRecoveryBootstrapPassphrase {
            passphrase: b"recovery-pass".to_vec(),
        })
        .await
        .unwrap();
    assert!(!done, "the ordinary share is still outstanding");

    let done = gates
        .ordinary
        .ask(HandleContributeBootstrapPassphrase {
            passphrase: b"ordinary-pass".to_vec(),
        })
        .await
        .unwrap();
    assert!(done, "both seats are filled, so bootstrap must finalize");
    assert_eq!(
        actors.vault.ask(GetState {}).await.unwrap(),
        VaultState::Unsealed
    );

    // Both peers hold the same id in their own table (asserted above), so only the attestation
    // kind tells the two envelopes apart. Two rows means the recovery peer signed as itself
    // rather than overwriting the ordinary operator's attestation.
    let kinds = common::eventually("both bootstrap attestations are written", || {
        let db = db.clone();
        async move {
            let mut conn = db.get().await.unwrap();
            let mut kinds: Vec<String> = schema::integrity_envelope::table
                .select(schema::integrity_envelope::entity_kind)
                .load(&mut conn)
                .await
                .unwrap();
            kinds.sort();
            (kinds.len() == 2).then_some(kinds)
        }
    })
    .await;
    assert_eq!(
        kinds,
        vec![
            "operator_credentials".to_owned(),
            "recovery_operator_credentials".to_owned(),
        ]
    );
}

/// Every gate action that belongs to one role refuses the other, and refuses it before the
/// action takes effect.
///
/// The vault is left unbootstrapped and the coordinator idle on purpose: an unguarded handler
/// would reach the vault or the coordinator and come back with `State`, `NotBootstrapping` or
/// `NotUnsealing`, so `RoleNotPermitted` can only come from the role check itself.
///
/// §3.4/§3.5: `HandleBootstrapEncryptedKey` matters most here. It hands the vault a root key of
/// the peer's choosing, and the window it needs -- an unbootstrapped vault that already holds
/// recovery identity rows -- is exactly the state committee formation has to pass through.
#[tokio::test]
#[test_log::test]
async fn vault_gate_refuses_the_actions_of_the_other_role() {
    let db = db::create_test_pool().await;
    let actors = common::spawn_actors(db.clone()).await;
    let gates = spawn_role_gates(&db, &actors).await;

    assert_role_refused(
        "a recovery operator declared the committee",
        gates
            .recovery
            .ask(HandleDeclareCommittee {
                count: 1,
                recovery_count: 1,
            })
            .await,
    );

    // A key the vault would have accepted, negotiated through the gate's own handshake -- so
    // the refusal comes from the role and not from a malformed request.
    let seized_key = bootstrap_key_for(&gates.recovery, b"recovery-seized-32-byte-seal-key").await;
    assert_role_refused(
        "a recovery operator bootstrapped the vault",
        gates.recovery.ask(seized_key).await,
    );

    assert_role_refused(
        "a recovery operator contributed an ordinary bootstrap share",
        gates
            .recovery
            .ask(HandleContributeBootstrapPassphrase {
                passphrase: b"forged-ordinary-pass".to_vec(),
            })
            .await,
    );

    assert_role_refused(
        "an ordinary operator contributed a recovery bootstrap share",
        gates
            .ordinary
            .ask(HandleContributeRecoveryBootstrapPassphrase {
                passphrase: b"forged-recovery-pass".to_vec(),
            })
            .await,
    );

    assert_role_refused(
        "a recovery operator contributed an ordinary unseal share",
        gates
            .recovery
            .ask(HandleContributeUnsealPassphrase {
                passphrase: b"forged-ordinary-pass".to_vec(),
            })
            .await,
    );

    assert_role_refused(
        "an ordinary operator contributed a recovery unseal share",
        gates
            .ordinary
            .ask(HandleContributeRecoveryUnsealPassphrase {
                passphrase: b"forged-recovery-pass".to_vec(),
            })
            .await,
    );

    // Nothing above took effect: the refusals came before the vault and the coordinator.
    assert_eq!(
        actors.vault.ask(GetState {}).await.unwrap(),
        VaultState::Unbootstrapped,
        "a refused request still reached the vault"
    );
}
