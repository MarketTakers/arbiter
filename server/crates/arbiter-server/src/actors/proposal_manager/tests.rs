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

/// A sleeping recovery electorate must not raise the bar for an ordinary proposal.
#[tokio::test]
async fn sleeping_recovery_operators_do_not_count_towards_quorum() {
    let id = ProposalId::from_raw(9);
    let voter = OperatorIdentityId::from_raw(3);
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
    // Two recovery operators exist but are asleep, so the threshold stays at 1 of 1.
    store.expect_tally().returning(|_| Ok(tally(1, 0, 1, 2)));
    store.expect_set_status().times(1).returning(|_, _| Ok(()));
    store.expect_load_kind().returning(|_, _| {
        Ok(crate::db::proposal::ProposalKind::ApproveSdkClient(
            crate::db::proposal::approve_sdk_client::Settings { client_id: 1 },
        ))
    });

    let mut manager =
        ProposalManager::with_store(Arc::new(store), GlobalActors::spawn_message_bus());

    let outcome = manager
        .cast_vote(id, voter, true, signature.to_bytes())
        .await
        .expect("a valid vote must be accepted");

    assert_eq!(outcome, VoteOutcome::Approved);
}
