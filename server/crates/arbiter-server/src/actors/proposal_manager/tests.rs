//! The quorum rules, exercised without a database.
//!
//! These assertions are the point of [`super::store::ProposalStore`]: until the actor took
//! its data through a trait, checking that two of three operators carry an ordinary
//! proposal meant opening SQLite and registering operators first.

use super::{
    ProposalManager, VoteOutcome,
    store::{MockProposalStore, Tally},
};
use crate::{
    actors::GlobalActors,
    crypto::governance::vote_message,
    db::{
        models::{OperatorIdentityId, Proposal, ProposalId, ProposalStatus, SqliteTimestamp},
        proposal::ProposalKindTag,
    },
};
use arbiter_crypto::authn::{SigningContext, SigningKey};
use chrono::{Duration, Utc};
use std::sync::Arc;

const fn tally(approve: i64, reject: i64, ordinary: i64, recovery: i64) -> Tally {
    Tally {
        approve,
        reject,
        total_ordinary: ordinary,
        total_recovery: recovery,
    }
}

#[test]
fn simple_majority_approves_at_two_of_three() {
    assert_eq!(
        ProposalManager::evaluate_quorum(&tally(2, 0, 3, 0), false),
        VoteOutcome::Approved
    );
}

#[test]
fn one_of_three_is_not_yet_a_majority() {
    assert_eq!(
        ProposalManager::evaluate_quorum(&tally(1, 0, 3, 0), false),
        VoteOutcome::Pending
    );
}

#[test]
fn full_quorum_kind_needs_every_voter() {
    assert_eq!(
        ProposalManager::evaluate_quorum(&tally(2, 0, 3, 0), true),
        VoteOutcome::Pending,
        "two of three must not carry a key-rotation proposal"
    );
}

#[test]
fn recovery_voters_count_towards_full_quorum() {
    assert_eq!(
        ProposalManager::evaluate_quorum(&tally(3, 0, 2, 1), true),
        VoteOutcome::Approved
    );
    assert_eq!(
        ProposalManager::evaluate_quorum(&tally(2, 0, 2, 1), true),
        VoteOutcome::Pending,
        "the sleeping recovery operator still owes a vote"
    );
}

#[test]
fn rejection_is_decided_once_approval_is_unreachable() {
    // Threshold is 2 of 3, so two rejections leave at most one approval available.
    assert_eq!(
        ProposalManager::evaluate_quorum(&tally(0, 2, 3, 0), false),
        VoteOutcome::Rejected
    );
    assert_eq!(
        ProposalManager::evaluate_quorum(&tally(0, 1, 3, 0), false),
        VoteOutcome::Pending,
        "one rejection still leaves two approvals reachable"
    );
}

#[test]
fn a_single_rejection_sinks_a_full_quorum_proposal() {
    assert_eq!(
        ProposalManager::evaluate_quorum(&tally(2, 1, 3, 0), true),
        VoteOutcome::Rejected
    );
}

fn pending_proposal(id: ProposalId, kind: ProposalKindTag) -> Proposal {
    let now = Utc::now();
    Proposal {
        id,
        kind,
        initiator_id: OperatorIdentityId::from_raw(1),
        created_at: SqliteTimestamp::from(now),
        expires_at: SqliteTimestamp::from(now + Duration::days(1)),
        status: ProposalStatus::Pending,
    }
}

/// The mock earns its keep here: reaching quorum must flip the stored status to
/// `Approved` exactly once. Signature verification stays real -- only the database is
/// stubbed out.
#[tokio::test]
async fn reaching_quorum_marks_the_proposal_approved() {
    let id = ProposalId::from_raw(1);
    let voter = OperatorIdentityId::from_raw(1);
    let key = SigningKey::generate();
    let signature = key
        .sign_message(&vote_message(id, true), SigningContext::GovernanceVote)
        .expect("signing a vote must succeed");
    let public_key = key.public_key().to_bytes();

    let mut store = MockProposalStore::new();
    store
        .expect_load()
        .returning(move |id| Ok(pending_proposal(id, ProposalKindTag::TriggerRekey)));
    store.expect_has_voted().returning(|_, _| Ok(false));
    store
        .expect_operator_public_key()
        .returning(move |_| Ok(public_key.clone()));
    store.expect_record_vote().returning(|_| Ok(()));
    store.expect_is_recovery_active().returning(|| Ok(false));
    store.expect_tally().returning(|_| Ok(tally(1, 0, 1, 0)));
    store
        .expect_set_status()
        .withf(move |got, status| *got == id && *status == ProposalStatus::Approved)
        .times(1)
        .returning(|_, _| Ok(()));
    store
        .expect_load_kind()
        .returning(|_, _| Ok(crate::db::proposal::ProposalKind::TriggerRekey));

    let mut manager =
        ProposalManager::with_store(Arc::new(store), GlobalActors::spawn_message_bus());

    let outcome = manager
        .cast_vote(id, voter, true, signature.to_bytes())
        .await
        .expect("a valid vote must be accepted");

    assert_eq!(outcome, VoteOutcome::Approved);
}

/// A vote that does not reach the threshold must leave the stored status alone.
#[tokio::test]
async fn a_vote_short_of_quorum_does_not_touch_the_status() {
    let id = ProposalId::from_raw(7);
    let voter = OperatorIdentityId::from_raw(2);
    let key = SigningKey::generate();
    let signature = key
        .sign_message(&vote_message(id, true), SigningContext::GovernanceVote)
        .expect("signing a vote must succeed");
    let public_key = key.public_key().to_bytes();

    let mut store = MockProposalStore::new();
    store
        .expect_load()
        .returning(move |id| Ok(pending_proposal(id, ProposalKindTag::ApproveSdkClient)));
    store.expect_has_voted().returning(|_, _| Ok(false));
    store
        .expect_operator_public_key()
        .returning(move |_| Ok(public_key.clone()));
    store.expect_record_vote().returning(|_| Ok(()));
    store.expect_is_recovery_active().returning(|| Ok(false));
    store.expect_tally().returning(|_| Ok(tally(1, 0, 3, 0)));
    store.expect_set_status().never();

    let mut manager =
        ProposalManager::with_store(Arc::new(store), GlobalActors::spawn_message_bus());

    let outcome = manager
        .cast_vote(id, voter, true, signature.to_bytes())
        .await
        .expect("a valid vote must be accepted");

    assert_eq!(outcome, VoteOutcome::Pending);
}

/// Drives one `cast_vote` on a proposal of the given `kind` through a mocked store and
/// returns the outcome. `recovery_active` decides what `is_recovery_active` reports;
/// `expected_status` is the status a settled outcome must be persisted under.
///
/// `set_status` carries an argument matcher but no `.times()`: whichever outcome a caller
/// asserts is either `Approved` or `Rejected` (never `Pending`), so the write must happen
/// with the right status if it happens at all, but leaving the count unconstrained means a
/// regression that turns the outcome into `Pending` still fails on the caller's own
/// `assert_eq!` -- a readable diff -- rather than on a mockall cardinality panic that hides
/// what the actor actually computed.
async fn settle_vote_with(
    kind: ProposalKindTag,
    tally: Tally,
    recovery_active: bool,
    expected_status: ProposalStatus,
) -> VoteOutcome {
    let id = ProposalId::from_raw(11);
    let voter = OperatorIdentityId::from_raw(1);
    let key = SigningKey::generate();
    let signature = key
        .sign_message(&vote_message(id, true), SigningContext::GovernanceVote)
        .expect("signing a vote must succeed");
    let public_key = key.public_key().to_bytes();

    let mut store = MockProposalStore::new();
    store
        .expect_load()
        .returning(move |id| Ok(pending_proposal(id, kind)));
    store.expect_has_voted().returning(|_, _| Ok(false));
    store
        .expect_operator_public_key()
        .returning(move |_| Ok(public_key.clone()));
    store.expect_record_vote().returning(|_| Ok(()));
    store
        .expect_is_recovery_active()
        .returning(move || Ok(recovery_active));
    store.expect_tally().returning(move |_| Ok(tally));
    store
        .expect_set_status()
        .withf(move |_, status| *status == expected_status)
        .returning(|_, _| Ok(()));
    store.expect_load_kind().returning(move |_, _| {
        Ok(match kind {
            ProposalKindTag::TriggerRekey => crate::db::proposal::ProposalKind::TriggerRekey,
            ProposalKindTag::ApproveSdkClient => {
                crate::db::proposal::ProposalKind::ApproveSdkClient(
                    crate::db::proposal::approve_sdk_client::Settings { client_id: 1 },
                )
            }
            ProposalKindTag::ReplaceOperator => crate::db::proposal::ProposalKind::ReplaceOperator(
                crate::db::proposal::replace_operator::Settings {
                    old_operator_id: OperatorIdentityId::from_raw(1),
                    new_pubkey: vec![0u8; 32],
                },
            ),
            other => unreachable!("settle_vote_with has no load_kind fixture for {other:?}"),
        })
    });

    let mut manager =
        ProposalManager::with_store(Arc::new(store), GlobalActors::spawn_message_bus());

    manager
        .cast_vote(id, voter, true, signature.to_bytes())
        .await
        .expect("a valid vote must be accepted")
}

/// The full-quorum rejection path is insensitive to electorate size by construction:
/// `threshold == total_eligible` there, so `total_eligible - threshold` is always 0 and any
/// single rejection settles the proposal, whether or not recovery operators are (wrongly)
/// counted. This does not exercise the electorate-narrowing fix -- see
/// `unanimous_ordinary_rejection_rejects_a_non_full_quorum_proposal_while_recovery_is_awake`
/// below for the test that does -- it just pins that `cast_vote` still writes `Rejected`
/// through `settle` for a full-quorum kind.
#[tokio::test]
async fn unanimous_rejection_settles_a_full_quorum_rekey_via_cast_vote() {
    let outcome = settle_vote_with(
        ProposalKindTag::TriggerRekey,
        Tally {
            approve: 0,
            reject: 3,
            total_ordinary: 3,
            total_recovery: 2,
        },
        /* recovery_active */ true,
        ProposalStatus::Rejected,
    )
    .await;

    assert_eq!(outcome, VoteOutcome::Rejected);
}

/// §3.3 full quorum for a rekey means every *ordinary* operator, not every identity on file.
#[tokio::test]
async fn unanimous_ordinary_approval_approves_a_rekey_while_recovery_is_awake() {
    let outcome = settle_vote_with(
        ProposalKindTag::TriggerRekey,
        Tally {
            approve: 3,
            reject: 0,
            total_ordinary: 3,
            total_recovery: 2,
        },
        /* recovery_active */ true,
        ProposalStatus::Approved,
    )
    .await;

    assert_eq!(outcome, VoteOutcome::Approved);
}

/// §3.5: recovery operators do not vote on `ApproveSdkClient`, so they must not inflate its
/// electorate. Before the fix, `total_eligible` counted them anyway (5, not 3), so the
/// rejection test `reject > total_eligible - threshold` became `3 > 5 - 2 = 3`, which is
/// false -- three unanimous rejections left the proposal `Pending` forever, since a fourth
/// vote could never arrive. After the fix, `total_eligible` is 3 and the same test becomes
/// `3 > 3 - 2 = 1`, which settles it.
#[tokio::test]
async fn unanimous_ordinary_rejection_rejects_a_non_full_quorum_proposal_while_recovery_is_awake() {
    let outcome = settle_vote_with(
        ProposalKindTag::ApproveSdkClient,
        Tally {
            approve: 0,
            reject: 3,
            total_ordinary: 3,
            total_recovery: 2,
        },
        /* recovery_active */ true,
        ProposalStatus::Rejected,
    )
    .await;

    assert_eq!(outcome, VoteOutcome::Rejected);
}

/// §3.5/§3.6: `ReplaceOperator` is the one kind recovery may vote on, so it is the only kind
/// where whether recovery is awake is observable at all -- for every other kind
/// `narrow_electorate` zeroes `total_recovery` regardless of `is_recovery_active`, short-
/// circuiting before that call. A sleeping recovery electorate must not raise the bar here:
/// with 1 ordinary operator and 2 (asleep) recovery operators, the lone ordinary approval
/// must already reach full quorum.
#[tokio::test]
async fn sleeping_recovery_operators_do_not_count_towards_quorum() {
    let outcome = settle_vote_with(
        ProposalKindTag::ReplaceOperator,
        Tally {
            approve: 1,
            reject: 0,
            total_ordinary: 1,
            total_recovery: 2,
        },
        /* recovery_active */ false,
        ProposalStatus::Approved,
    )
    .await;

    assert_eq!(outcome, VoteOutcome::Approved);
}
