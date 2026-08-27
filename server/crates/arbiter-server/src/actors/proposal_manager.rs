use crate::{
    actors::proposal_manager::{
        events::ProposalApproved,
        store::{DieselProposalStore, ProposalStore, Tally},
    },
    crypto::governance,
    db::{
        self,
        models::{
            NewProposalVote, NewRecoveryProposalVote, OperatorIdentityId, Proposal, ProposalId,
            ProposalStatus, RecoveryOperatorIdentityId, SqliteTimestamp,
        },
        proposal::{ProposalKind, ProposalKindTag},
    },
};
use chrono::Utc;
use kameo::{Actor, actor::ActorRef, messages};
use kameo_actors::message_bus::{MessageBus, Publish};
use std::sync::Arc;
use tracing::warn;

pub mod events;
pub mod store;

pub const DEFAULT_TTL_SECS: u32 = 7 * 24 * 60 * 60; // 7 days
pub const MAX_TTL_SECS: u32 = DEFAULT_TTL_SECS;

/// Recovery operators stay asleep for this long after a wake-up is requested, so the other
/// operators have time to dispute it (§3.6).
const WAKEUP_DELAY_SECS: i32 = 14 * 24 * 60 * 60; // 14 days

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteOutcome {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Proposal not found")]
    ProposalNotFound,
    #[error("Proposal is not pending")]
    ProposalNotPending,
    #[error("Proposal has expired")]
    ProposalExpired,
    #[error("Requested TTL exceeds the maximum of {} seconds", MAX_TTL_SECS)]
    TtlTooLong,
    #[error("Operator already voted on this proposal")]
    AlreadyVoted,
    #[error("Invalid vote signature")]
    InvalidSignature,
    #[error("Operator not found")]
    OperatorNotFound,
    #[error("Database connection error: {0}")]
    DatabaseConnection(#[from] db::PoolError),
    #[error("Database query error: {0}")]
    DatabaseQuery(#[from] diesel::result::Error),
    #[error("Proposal manager is unavailable")]
    Unavailable,
    #[error("Recovery operators are sleeping")]
    RecoveryNotActive,
    #[error("Recovery operators may only vote on operator replacement")]
    NotAllowedForRecoveryOperator,
    #[error("A recovery wake-up is already pending or active")]
    WakeupAlreadyPending,
    #[error("No active recovery wake-up to cancel")]
    NoActiveWakeup,
}

#[derive(Debug)]
pub struct ProposalSummary {
    pub id: ProposalId,
    pub kind: ProposalKindTag,
    pub initiator_id: OperatorIdentityId,
    pub expires_at: SqliteTimestamp,
    pub approve_count: i64,
    pub reject_count: i64,
}

#[derive(Actor)]
pub struct ProposalManager {
    pub(crate) store: Arc<dyn ProposalStore>,
    pub(crate) events: ActorRef<MessageBus>,
}

impl ProposalManager {
    pub fn new(db: db::DatabasePool, events: ActorRef<MessageBus>) -> Self {
        Self::with_store(Arc::new(DieselProposalStore::new(db)), events)
    }

    /// Builds the actor over an arbitrary store, so tests can supply a mock.
    pub(crate) const fn with_store(
        store: Arc<dyn ProposalStore>,
        events: ActorRef<MessageBus>,
    ) -> Self {
        Self { store, events }
    }
}

#[messages]
impl ProposalManager {
    #[message]
    pub async fn create_proposal(
        &mut self,
        kind: ProposalKind,
        initiator_id: OperatorIdentityId,
        ttl_secs: Option<u32>,
    ) -> Result<ProposalId, Error> {
        let ttl = ttl_secs.unwrap_or(DEFAULT_TTL_SECS);
        if ttl > MAX_TTL_SECS {
            return Err(Error::TtlTooLong);
        }
        let expires_at =
            SqliteTimestamp::from(Utc::now() + chrono::Duration::seconds(i64::from(ttl)));

        self.store.create(kind, initiator_id, expires_at).await
    }

    #[message]
    pub async fn query_pending(&mut self, operator_id: OperatorIdentityId) -> Vec<ProposalSummary> {
        self.store
            .pending_for(operator_id)
            .await
            .unwrap_or_else(|e| {
                warn!(?e, "query_pending failed");
                vec![]
            })
    }

    #[message]
    pub async fn cast_vote(
        &mut self,
        proposal_id: ProposalId,
        operator_id: OperatorIdentityId,
        approve: bool,
        signature: Vec<u8>,
    ) -> Result<VoteOutcome, Error> {
        let proposal = self.store.load(proposal_id).await?;

        // Checked before the status check so AlreadyVoted takes priority.
        if self.store.has_voted(proposal_id, operator_id).await? {
            return Err(Error::AlreadyVoted);
        }

        Self::check_votable(&proposal)?;

        let public_key = self.store.operator_public_key(operator_id).await?;
        governance::verify_vote(&public_key, proposal_id, approve, &signature)
            .map_err(|_| Error::InvalidSignature)?;

        self.store
            .record_vote(NewProposalVote {
                proposal_id,
                operator_id,
                approve,
                signature,
            })
            .await?;

        let mut tally = self.store.tally(proposal_id).await?;
        // §3.5: recovery operators only join the electorate once they are awake.
        if !self.store.is_recovery_active().await? {
            tally.total_recovery = 0;
        }

        self.settle(&proposal, &tally).await
    }

    /// §3.6: Any ordinary operator may request recovery wake-up.
    /// Fails if a wake-up is already pending or active.
    #[message]
    pub async fn request_recovery_wakeup(
        &mut self,
        operator_id: OperatorIdentityId,
    ) -> Result<(), Error> {
        if self.store.has_uncancelled_wakeup().await? {
            return Err(Error::WakeupAlreadyPending);
        }
        self.store.request_wakeup(operator_id).await
    }

    /// §3.6: Any ordinary operator may cancel a pending wake-up request.
    /// Fails if there is no uncancelled request.
    #[message]
    pub async fn cancel_recovery_wakeup(
        &mut self,
        operator_id: OperatorIdentityId,
    ) -> Result<(), Error> {
        if self.store.cancel_wakeup(operator_id).await? {
            Ok(())
        } else {
            Err(Error::NoActiveWakeup)
        }
    }

    /// §3.5: Recovery operators may only vote on operator replacement proposals.
    /// §3.6: Voting is gated behind recovery being active (14-day window elapsed).
    #[message]
    pub async fn cast_recovery_vote(
        &mut self,
        proposal_id: ProposalId,
        recovery_operator_id: RecoveryOperatorIdentityId,
        approve: bool,
        signature: Vec<u8>,
    ) -> Result<VoteOutcome, Error> {
        let proposal = self.store.load(proposal_id).await?;

        if proposal.kind != ProposalKindTag::ReplaceOperator {
            return Err(Error::NotAllowedForRecoveryOperator);
        }

        if !self.store.is_recovery_active().await? {
            return Err(Error::RecoveryNotActive);
        }

        if self
            .store
            .has_recovery_voted(proposal_id, recovery_operator_id)
            .await?
        {
            return Err(Error::AlreadyVoted);
        }

        Self::check_votable(&proposal)?;

        let public_key = self
            .store
            .recovery_operator_public_key(recovery_operator_id)
            .await?;
        governance::verify_vote(&public_key, proposal_id, approve, &signature)
            .map_err(|_| Error::InvalidSignature)?;

        self.store
            .record_recovery_vote(NewRecoveryProposalVote {
                proposal_id,
                recovery_operator_id,
                approve,
                signature,
            })
            .await?;

        let tally = self.store.tally(proposal_id).await?;
        self.settle(&proposal, &tally).await
    }
}

impl ProposalManager {
    /// A vote only counts while the proposal is still open.
    fn check_votable(proposal: &Proposal) -> Result<(), Error> {
        if proposal.status != ProposalStatus::Pending {
            return Err(Error::ProposalNotPending);
        }
        if proposal.expires_at.0 <= Utc::now() {
            return Err(Error::ProposalExpired);
        }
        Ok(())
    }

    /// Pure quorum arithmetic — no I/O, so the rules can be tested directly (§3.3).
    ///
    /// A proposal is rejected once approval has become unreachable: even if every voter
    /// who has not spoken yet approved, the threshold could not be met.
    #[must_use]
    pub(crate) const fn evaluate_quorum(tally: &Tally, requires_full_quorum: bool) -> VoteOutcome {
        let total_eligible = tally.total_ordinary + tally.total_recovery;

        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_possible_wrap,
            clippy::as_conversions,
            reason = "operator counts are always small positive integers"
        )]
        // §3.3: key-rotation proposals require every eligible voter to approve.
        // §3.5: when recovery is active, recovery operators are eligible too.
        let threshold: i64 = if requires_full_quorum {
            total_eligible
        } else {
            crate::crypto::shamir::shamir_threshold(tally.total_ordinary as usize) as i64
        };

        if tally.approve >= threshold {
            VoteOutcome::Approved
        } else if tally.reject > total_eligible - threshold {
            VoteOutcome::Rejected
        } else {
            VoteOutcome::Pending
        }
    }

    /// Applies the quorum rules to a fresh tally and records whatever they decide.
    async fn settle(&self, proposal: &Proposal, tally: &Tally) -> Result<VoteOutcome, Error> {
        let outcome = Self::evaluate_quorum(tally, proposal.kind.requires_full_quorum());

        match outcome {
            VoteOutcome::Approved => self.announce_approval(proposal).await?,
            VoteOutcome::Rejected => {
                self.store
                    .set_status(proposal.id, ProposalStatus::Rejected)
                    .await?;
            }
            VoteOutcome::Pending => {}
        }

        Ok(outcome)
    }

    /// Marks the proposal approved and hands the outcome to whoever owns that kind.
    ///
    /// The outcome is published, not executed: this actor coordinates voting and nothing
    /// else. Executors subscribe on the bus, so a vote is answered once the quorum is
    /// recorded rather than once the effect has landed.
    async fn announce_approval(&self, proposal: &Proposal) -> Result<(), Error> {
        self.store
            .set_status(proposal.id, ProposalStatus::Approved)
            .await?;

        let kind = self.store.load_kind(proposal.id, proposal.kind).await?;
        let _ = self
            .events
            .tell(Publish(ProposalApproved {
                id: proposal.id,
                kind,
            }))
            .await;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
