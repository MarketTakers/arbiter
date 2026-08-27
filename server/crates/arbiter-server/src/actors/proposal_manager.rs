use crate::{
    actors::proposal_manager::events::ProposalApproved,
    crypto::governance,
    db::{
        self,
        functions::unixepoch,
        models::{
            NewProposal, NewProposalVote, NewRecoveryProposalVote, NewRecoveryWakeupRequest,
            OperatorIdentityId, Proposal, ProposalId, ProposalStatus, RecoveryOperatorIdentityId,
            SqliteTimestamp,
        },
        proposal::{ProposalKind, ProposalKindTag},
        schema,
    },
};
use chrono::Utc;
use diesel::{
    ExpressionMethods as _, QueryDsl,
    dsl::{exists, select},
};
use diesel_async::{AsyncConnection as _, RunQueryDsl};
use kameo::{Actor, actor::ActorRef, messages};
use kameo_actors::message_bus::{MessageBus, Publish};
use std::collections::HashMap;
use strum::IntoDiscriminant as _;
use tracing::warn;

pub mod events;

pub const DEFAULT_TTL_SECS: u32 = 7 * 24 * 60 * 60; // 7 days
pub const MAX_TTL_SECS: u32 = DEFAULT_TTL_SECS;

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
    pub(crate) db: db::DatabasePool,
    pub(crate) events: ActorRef<MessageBus>,
}

impl ProposalManager {
    pub const fn new(db: db::DatabasePool, events: ActorRef<MessageBus>) -> Self {
        Self { db, events }
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

        let id: ProposalId = self
            .db
            .get()
            .await?
            .transaction(async |conn| {
                let id: ProposalId = diesel::insert_into(schema::proposal::table)
                    .values(&NewProposal {
                        kind: kind.discriminant(),
                        initiator_id,
                        expires_at,
                    })
                    .returning(schema::proposal::id)
                    .get_result(conn)
                    .await?;
                db::proposal::insert_kind(conn, id, &kind).await?;
                Ok::<_, diesel::result::Error>(id)
            })
            .await?;

        Ok(id)
    }

    #[message]
    pub async fn query_pending(&mut self, operator_id: OperatorIdentityId) -> Vec<ProposalSummary> {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::as_conversions,
            reason = "fixme! #84; this will break in 2038"
        )]
        let now_ts = Utc::now().timestamp() as i32;

        let Ok(mut conn) = self.db.get().await else {
            warn!("query_pending: failed to acquire DB connection");
            return vec![];
        };

        let voted_ids: Vec<ProposalId> = schema::proposal_vote::table
            .filter(schema::proposal_vote::operator_id.eq(operator_id))
            .select(schema::proposal_vote::proposal_id)
            .load(&mut conn)
            .await
            .unwrap_or_default();

        let proposals: Vec<Proposal> = schema::proposal::table
            .filter(schema::proposal::status.eq(ProposalStatus::Pending))
            .filter(schema::proposal::expires_at.gt(now_ts))
            .filter(diesel::dsl::not(schema::proposal::id.eq_any(&voted_ids)))
            .load(&mut conn)
            .await
            .unwrap_or_default();

        let ids: Vec<ProposalId> = proposals.iter().map(|p| p.id).collect();
        let tallies: Vec<(ProposalId, bool, i64)> = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq_any(&ids))
            .group_by((
                schema::proposal_vote::proposal_id,
                schema::proposal_vote::approve,
            ))
            .select((
                schema::proposal_vote::proposal_id,
                schema::proposal_vote::approve,
                diesel::dsl::count_star(),
            ))
            .load(&mut conn)
            .await
            .unwrap_or_default();

        let mut by_proposal: HashMap<ProposalId, (i64, i64)> = HashMap::new();
        for (proposal_id, approve, count) in tallies {
            let entry = by_proposal.entry(proposal_id).or_insert((0, 0));
            if approve {
                entry.0 += count;
            } else {
                entry.1 += count;
            }
        }

        proposals
            .into_iter()
            .map(|p| {
                let (approve_count, reject_count) =
                    by_proposal.get(&p.id).copied().unwrap_or((0, 0));
                ProposalSummary {
                    id: p.id,
                    kind: p.kind,
                    initiator_id: p.initiator_id,
                    expires_at: p.expires_at,
                    approve_count,
                    reject_count,
                }
            })
            .collect()
    }

    #[message]
    pub async fn cast_vote(
        &mut self,
        proposal_id: ProposalId,
        operator_id: OperatorIdentityId,
        approve: bool,
        signature: Vec<u8>,
    ) -> Result<VoteOutcome, Error> {
        let mut conn = self.db.get().await?;

        // Load proposal — must exist
        let proposal: Proposal = schema::proposal::table
            .find(proposal_id)
            .first(&mut conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::ProposalNotFound,
                other => Error::DatabaseQuery(other),
            })?;

        // Check for duplicate vote before status check so AlreadyVoted takes priority
        let already_voted: bool = select(exists(
            schema::proposal_vote::table
                .filter(schema::proposal_vote::proposal_id.eq(proposal_id))
                .filter(schema::proposal_vote::operator_id.eq(operator_id)),
        ))
        .get_result(&mut conn)
        .await?;
        if already_voted {
            return Err(Error::AlreadyVoted);
        }

        if proposal.status != ProposalStatus::Pending {
            return Err(Error::ProposalNotPending);
        }

        if proposal.expires_at.0 <= Utc::now() {
            return Err(Error::ProposalExpired);
        }

        // Load operator public key from operator_identity
        let pubkey_bytes: Vec<u8> = schema::operator_identity::table
            .find(operator_id)
            .select(schema::operator_identity::public_key)
            .first(&mut conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::OperatorNotFound,
                other => Error::DatabaseQuery(other),
            })?;

        governance::verify_vote(&pubkey_bytes, proposal_id, approve, &signature)
            .map_err(|_| Error::InvalidSignature)?;

        // Insert vote
        diesel::insert_into(schema::proposal_vote::table)
            .values(&NewProposalVote {
                proposal_id,
                operator_id,
                approve,
                signature,
            })
            .execute(&mut conn)
            .await?;

        // Quorum check
        let total_operators: i64 = schema::operator_identity::table
            .count()
            .get_result(&mut conn)
            .await?;
        let recovery_active = Self::is_recovery_active_conn(&mut conn).await?;
        let total_recovery: i64 = if recovery_active {
            schema::recovery_operator_identity::table
                .count()
                .get_result(&mut conn)
                .await?
        } else {
            0
        };
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::as_conversions,
            reason = "operator count is always a small positive integer"
        )]
        let threshold = if proposal.kind.requires_full_quorum() {
            // §3.3: key-rotation proposals require every eligible voter to approve
            // §3.5: when recovery is active, recovery operators also vote on replace_operator
            (total_operators + total_recovery) as usize
        } else {
            crate::crypto::shamir::shamir_threshold(total_operators as usize)
        };

        let ordinary_approve: i64 = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::proposal_vote::approve.eq(true))
            .count()
            .get_result(&mut conn)
            .await?;
        let recovery_approve: i64 = schema::recovery_proposal_vote::table
            .filter(schema::recovery_proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::recovery_proposal_vote::approve.eq(true))
            .count()
            .get_result(&mut conn)
            .await?;
        let approve_count = ordinary_approve + recovery_approve;

        let ordinary_reject: i64 = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::proposal_vote::approve.eq(false))
            .count()
            .get_result(&mut conn)
            .await?;
        let recovery_reject: i64 = schema::recovery_proposal_vote::table
            .filter(schema::recovery_proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::recovery_proposal_vote::approve.eq(false))
            .count()
            .get_result(&mut conn)
            .await?;
        let reject_count = ordinary_reject + recovery_reject;

        #[expect(
            clippy::cast_possible_wrap,
            clippy::as_conversions,
            reason = "threshold is derived from operator count, always fits i64"
        )]
        let threshold_i64 = threshold as i64;

        if approve_count >= threshold_i64 {
            self.announce_approval(&mut conn, &proposal).await?;
            return Ok(VoteOutcome::Approved);
        }

        let total_eligible = total_operators + total_recovery;
        if reject_count > total_eligible - threshold_i64 {
            diesel::update(schema::proposal::table.find(proposal_id))
                .set(schema::proposal::status.eq(ProposalStatus::Rejected))
                .execute(&mut conn)
                .await?;
            return Ok(VoteOutcome::Rejected);
        }

        Ok(VoteOutcome::Pending)
    }

    /// §3.6: Any ordinary operator may request recovery wake-up.
    /// Fails if a wake-up is already pending or active.
    #[message]
    pub async fn request_recovery_wakeup(
        &mut self,
        operator_id: OperatorIdentityId,
    ) -> Result<(), Error> {
        let mut conn = self.db.get().await?;
        if Self::has_uncancelled_wakeup(&mut conn).await? {
            return Err(Error::WakeupAlreadyPending);
        }
        diesel::insert_into(schema::recovery_wakeup_request::table)
            .values(&NewRecoveryWakeupRequest {
                requested_by: operator_id,
            })
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    /// §3.6: Any ordinary operator may cancel a pending wake-up request.
    /// Fails if there is no uncancelled request.
    #[message]
    pub async fn cancel_recovery_wakeup(
        &mut self,
        operator_id: OperatorIdentityId,
    ) -> Result<(), Error> {
        let mut conn = self.db.get().await?;
        let rows_updated = diesel::update(schema::recovery_wakeup_request::table)
            .filter(schema::recovery_wakeup_request::cancelled_at.is_null())
            .set((
                schema::recovery_wakeup_request::cancelled_by.eq(Some(operator_id)),
                schema::recovery_wakeup_request::cancelled_at.eq(Some(SqliteTimestamp::now())),
            ))
            .execute(&mut conn)
            .await?;
        if rows_updated == 0 {
            return Err(Error::NoActiveWakeup);
        }
        Ok(())
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
        let mut conn = self.db.get().await?;

        let proposal: Proposal = schema::proposal::table
            .find(proposal_id)
            .first(&mut conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::ProposalNotFound,
                other => Error::DatabaseQuery(other),
            })?;

        if proposal.kind != ProposalKindTag::ReplaceOperator {
            return Err(Error::NotAllowedForRecoveryOperator);
        }

        if !Self::is_recovery_active_conn(&mut conn).await? {
            return Err(Error::RecoveryNotActive);
        }

        let already_voted: bool = select(exists(
            schema::recovery_proposal_vote::table
                .filter(schema::recovery_proposal_vote::proposal_id.eq(proposal_id))
                .filter(
                    schema::recovery_proposal_vote::recovery_operator_id.eq(recovery_operator_id),
                ),
        ))
        .get_result(&mut conn)
        .await?;
        if already_voted {
            return Err(Error::AlreadyVoted);
        }

        if proposal.status != ProposalStatus::Pending {
            return Err(Error::ProposalNotPending);
        }

        if proposal.expires_at.0 <= Utc::now() {
            return Err(Error::ProposalExpired);
        }

        let pubkey_bytes: Vec<u8> = schema::recovery_operator_identity::table
            .find(recovery_operator_id)
            .select(schema::recovery_operator_identity::public_key)
            .first(&mut conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::OperatorNotFound,
                other => Error::DatabaseQuery(other),
            })?;

        governance::verify_vote(&pubkey_bytes, proposal_id, approve, &signature)
            .map_err(|_| Error::InvalidSignature)?;

        diesel::insert_into(schema::recovery_proposal_vote::table)
            .values(&NewRecoveryProposalVote {
                proposal_id,
                recovery_operator_id,
                approve,
                signature,
            })
            .execute(&mut conn)
            .await?;

        // Quorum: all ordinary + all recovery operators must approve (§3.3 + §3.5)
        let total_ordinary: i64 = schema::operator_identity::table
            .count()
            .get_result(&mut conn)
            .await?;
        let total_recovery: i64 = schema::recovery_operator_identity::table
            .count()
            .get_result(&mut conn)
            .await?;
        let threshold_i64 = total_ordinary + total_recovery;

        let ordinary_approve: i64 = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::proposal_vote::approve.eq(true))
            .count()
            .get_result(&mut conn)
            .await?;
        let recovery_approve: i64 = schema::recovery_proposal_vote::table
            .filter(schema::recovery_proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::recovery_proposal_vote::approve.eq(true))
            .count()
            .get_result(&mut conn)
            .await?;
        let approve_count = ordinary_approve + recovery_approve;

        if approve_count >= threshold_i64 {
            self.announce_approval(&mut conn, &proposal).await?;
            return Ok(VoteOutcome::Approved);
        }

        let recovery_reject: i64 = schema::recovery_proposal_vote::table
            .filter(schema::recovery_proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::recovery_proposal_vote::approve.eq(false))
            .count()
            .get_result(&mut conn)
            .await?;
        let ordinary_reject: i64 = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::proposal_vote::approve.eq(false))
            .count()
            .get_result(&mut conn)
            .await?;
        let reject_count = ordinary_reject + recovery_reject;

        if reject_count > threshold_i64 - approve_count - reject_count {
            diesel::update(schema::proposal::table.find(proposal_id))
                .set(schema::proposal::status.eq(ProposalStatus::Rejected))
                .execute(&mut conn)
                .await?;
            return Ok(VoteOutcome::Rejected);
        }

        Ok(VoteOutcome::Pending)
    }
}

impl ProposalManager {
    const WAKEUP_DELAY_SECS: i32 = 14 * 24 * 60 * 60;

    /// Returns true when an uncancelled wakeup request has passed the 14-day dispute window.
    async fn is_recovery_active_conn(conn: &mut db::DatabaseConnection) -> Result<bool, Error> {
        select(exists(
            schema::recovery_wakeup_request::table
                .filter(schema::recovery_wakeup_request::cancelled_at.is_null())
                .filter(
                    schema::recovery_wakeup_request::requested_at
                        .le(unixepoch("now") - Self::WAKEUP_DELAY_SECS),
                ),
        ))
        .get_result(conn)
        .await
        .map_err(Error::from)
    }

    /// Returns true when there is any uncancelled wakeup request (pending or active).
    async fn has_uncancelled_wakeup(conn: &mut db::DatabaseConnection) -> Result<bool, Error> {
        select(exists(schema::recovery_wakeup_request::table.filter(
            schema::recovery_wakeup_request::cancelled_at.is_null(),
        )))
        .get_result(conn)
        .await
        .map_err(Error::from)
    }

    /// Marks the proposal approved and hands the outcome to whoever owns that kind.
    ///
    /// The outcome is published, not executed: this actor coordinates voting and nothing
    /// else. Executors subscribe on the bus, so a vote is answered once the quorum is
    /// recorded rather than once the effect has landed.
    async fn announce_approval(
        &self,
        conn: &mut db::DatabaseConnection,
        proposal: &Proposal,
    ) -> Result<(), Error> {
        diesel::update(schema::proposal::table.find(proposal.id))
            .set(schema::proposal::status.eq(ProposalStatus::Approved))
            .execute(conn)
            .await?;

        let kind = db::proposal::load_kind(conn, proposal.id, proposal.kind).await?;
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
