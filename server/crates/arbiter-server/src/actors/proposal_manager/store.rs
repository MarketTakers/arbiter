//! Database access for [`super::ProposalManager`], behind a trait.
//!
//! The actor holds a `dyn ProposalStore` rather than a connection pool, so the quorum
//! rules can be exercised against a mock instead of a live SQLite file.

use super::{Error, ProposalSummary, WAKEUP_DELAY_SECS};
use crate::db::{
    self,
    functions::unixepoch,
    models::{
        NewProposal, NewProposalVote, NewRecoveryProposalVote, NewRecoveryWakeupRequest,
        OperatorIdentityId, Proposal, ProposalId, ProposalStatus, RecoveryOperatorIdentityId,
        SqliteTimestamp,
    },
    proposal::{ProposalKind, ProposalKindTag},
    schema,
};

use async_trait::async_trait;
use chrono::Utc;
use diesel::{
    ExpressionMethods as _, QueryDsl,
    dsl::{exists, select},
};
use diesel_async::{AsyncConnection as _, RunQueryDsl};
use std::collections::HashMap;
use strum::IntoDiscriminant as _;

/// Everything the quorum rules need to know about one proposal's votes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tally {
    pub approve: i64,
    pub reject: i64,
    pub total_ordinary: i64,
    pub total_recovery: i64,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProposalStore: Send + Sync + 'static {
    /// Writes the proposal and its kind-specific rows in one transaction.
    async fn create(
        &self,
        kind: ProposalKind,
        initiator_id: OperatorIdentityId,
        expires_at: SqliteTimestamp,
    ) -> Result<ProposalId, Error>;

    async fn load(&self, id: ProposalId) -> Result<Proposal, Error>;

    async fn load_kind(&self, id: ProposalId, tag: ProposalKindTag) -> Result<ProposalKind, Error>;

    async fn has_voted(
        &self,
        id: ProposalId,
        operator_id: OperatorIdentityId,
    ) -> Result<bool, Error>;

    async fn has_recovery_voted(
        &self,
        id: ProposalId,
        recovery_operator_id: RecoveryOperatorIdentityId,
    ) -> Result<bool, Error>;

    async fn operator_public_key(&self, id: OperatorIdentityId) -> Result<Vec<u8>, Error>;

    async fn recovery_operator_public_key(
        &self,
        id: RecoveryOperatorIdentityId,
    ) -> Result<Vec<u8>, Error>;

    async fn record_vote(&self, vote: NewProposalVote) -> Result<(), Error>;

    async fn record_recovery_vote(&self, vote: NewRecoveryProposalVote) -> Result<(), Error>;

    /// Vote counts for one proposal, alongside the size of each electorate.
    async fn tally(&self, id: ProposalId) -> Result<Tally, Error>;

    async fn set_status(&self, id: ProposalId, status: ProposalStatus) -> Result<(), Error>;

    /// Pending, unexpired proposals this operator has not voted on yet.
    async fn pending_for(
        &self,
        operator_id: OperatorIdentityId,
    ) -> Result<Vec<ProposalSummary>, Error>;

    /// True once an uncancelled wake-up request has outlived the dispute window.
    async fn is_recovery_active(&self) -> Result<bool, Error>;

    /// True while any wake-up request stands, whether or not the window has elapsed.
    async fn has_uncancelled_wakeup(&self) -> Result<bool, Error>;

    async fn request_wakeup(&self, operator_id: OperatorIdentityId) -> Result<(), Error>;

    /// Returns false when there was no uncancelled request to cancel.
    async fn cancel_wakeup(&self, operator_id: OperatorIdentityId) -> Result<bool, Error>;
}

pub struct DieselProposalStore {
    db: db::DatabasePool,
}

impl DieselProposalStore {
    pub const fn new(db: db::DatabasePool) -> Self {
        Self { db }
    }
}

/// `NotFound` means the row is absent, which every caller reports as its own error.
fn missing(absent: Error) -> impl FnOnce(diesel::result::Error) -> Error {
    move |e| match e {
        diesel::result::Error::NotFound => absent,
        other => Error::DatabaseQuery(other),
    }
}

#[async_trait]
impl ProposalStore for DieselProposalStore {
    async fn create(
        &self,
        kind: ProposalKind,
        initiator_id: OperatorIdentityId,
        expires_at: SqliteTimestamp,
    ) -> Result<ProposalId, Error> {
        let id = self
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

    async fn load(&self, id: ProposalId) -> Result<Proposal, Error> {
        let mut conn = self.db.get().await?;
        schema::proposal::table
            .find(id)
            .first(&mut conn)
            .await
            .map_err(missing(Error::ProposalNotFound))
    }

    async fn load_kind(&self, id: ProposalId, tag: ProposalKindTag) -> Result<ProposalKind, Error> {
        let mut conn = self.db.get().await?;
        db::proposal::load_kind(&mut conn, id, tag)
            .await
            .map_err(Error::from)
    }

    async fn has_voted(
        &self,
        id: ProposalId,
        operator_id: OperatorIdentityId,
    ) -> Result<bool, Error> {
        let mut conn = self.db.get().await?;
        select(exists(
            schema::proposal_vote::table
                .filter(schema::proposal_vote::proposal_id.eq(id))
                .filter(schema::proposal_vote::operator_id.eq(operator_id)),
        ))
        .get_result(&mut conn)
        .await
        .map_err(Error::from)
    }

    async fn has_recovery_voted(
        &self,
        id: ProposalId,
        recovery_operator_id: RecoveryOperatorIdentityId,
    ) -> Result<bool, Error> {
        let mut conn = self.db.get().await?;
        select(exists(
            schema::recovery_proposal_vote::table
                .filter(schema::recovery_proposal_vote::proposal_id.eq(id))
                .filter(
                    schema::recovery_proposal_vote::recovery_operator_id.eq(recovery_operator_id),
                ),
        ))
        .get_result(&mut conn)
        .await
        .map_err(Error::from)
    }

    async fn operator_public_key(&self, id: OperatorIdentityId) -> Result<Vec<u8>, Error> {
        let mut conn = self.db.get().await?;
        schema::operator_identity::table
            .find(id)
            .select(schema::operator_identity::public_key)
            .first(&mut conn)
            .await
            .map_err(missing(Error::OperatorNotFound))
    }

    async fn recovery_operator_public_key(
        &self,
        id: RecoveryOperatorIdentityId,
    ) -> Result<Vec<u8>, Error> {
        let mut conn = self.db.get().await?;
        schema::recovery_operator_identity::table
            .find(id)
            .select(schema::recovery_operator_identity::public_key)
            .first(&mut conn)
            .await
            .map_err(missing(Error::OperatorNotFound))
    }

    async fn record_vote(&self, vote: NewProposalVote) -> Result<(), Error> {
        let mut conn = self.db.get().await?;
        diesel::insert_into(schema::proposal_vote::table)
            .values(&vote)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    async fn record_recovery_vote(&self, vote: NewRecoveryProposalVote) -> Result<(), Error> {
        let mut conn = self.db.get().await?;
        diesel::insert_into(schema::recovery_proposal_vote::table)
            .values(&vote)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    async fn tally(&self, id: ProposalId) -> Result<Tally, Error> {
        let mut conn = self.db.get().await?;

        let ordinary_approve: i64 = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq(id))
            .filter(schema::proposal_vote::approve.eq(true))
            .count()
            .get_result(&mut conn)
            .await?;
        let recovery_approve: i64 = schema::recovery_proposal_vote::table
            .filter(schema::recovery_proposal_vote::proposal_id.eq(id))
            .filter(schema::recovery_proposal_vote::approve.eq(true))
            .count()
            .get_result(&mut conn)
            .await?;

        let ordinary_reject: i64 = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq(id))
            .filter(schema::proposal_vote::approve.eq(false))
            .count()
            .get_result(&mut conn)
            .await?;
        let recovery_reject: i64 = schema::recovery_proposal_vote::table
            .filter(schema::recovery_proposal_vote::proposal_id.eq(id))
            .filter(schema::recovery_proposal_vote::approve.eq(false))
            .count()
            .get_result(&mut conn)
            .await?;

        let total_ordinary: i64 = schema::operator_identity::table
            .count()
            .get_result(&mut conn)
            .await?;
        let total_recovery: i64 = schema::recovery_operator_identity::table
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Tally {
            approve: ordinary_approve + recovery_approve,
            reject: ordinary_reject + recovery_reject,
            total_ordinary,
            total_recovery,
        })
    }

    async fn set_status(&self, id: ProposalId, status: ProposalStatus) -> Result<(), Error> {
        let mut conn = self.db.get().await?;
        diesel::update(schema::proposal::table.find(id))
            .set(schema::proposal::status.eq(status))
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    async fn pending_for(
        &self,
        operator_id: OperatorIdentityId,
    ) -> Result<Vec<ProposalSummary>, Error> {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::as_conversions,
            reason = "fixme! #84; this will break in 2038"
        )]
        let now_ts = Utc::now().timestamp() as i32;

        let mut conn = self.db.get().await?;

        let voted_ids: Vec<ProposalId> = schema::proposal_vote::table
            .filter(schema::proposal_vote::operator_id.eq(operator_id))
            .select(schema::proposal_vote::proposal_id)
            .load(&mut conn)
            .await?;

        let proposals: Vec<Proposal> = schema::proposal::table
            .filter(schema::proposal::status.eq(ProposalStatus::Pending))
            .filter(schema::proposal::expires_at.gt(now_ts))
            .filter(diesel::dsl::not(schema::proposal::id.eq_any(&voted_ids)))
            .load(&mut conn)
            .await?;

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
            .await?;

        let mut by_proposal: HashMap<ProposalId, (i64, i64)> = HashMap::new();
        for (proposal_id, approve, count) in tallies {
            let entry = by_proposal.entry(proposal_id).or_insert((0, 0));
            if approve {
                entry.0 += count;
            } else {
                entry.1 += count;
            }
        }

        Ok(proposals
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
            .collect())
    }

    async fn is_recovery_active(&self) -> Result<bool, Error> {
        let mut conn = self.db.get().await?;
        select(exists(
            schema::recovery_wakeup_request::table
                .filter(schema::recovery_wakeup_request::cancelled_at.is_null())
                .filter(
                    schema::recovery_wakeup_request::requested_at
                        .le(unixepoch("now") - WAKEUP_DELAY_SECS),
                ),
        ))
        .get_result(&mut conn)
        .await
        .map_err(Error::from)
    }

    async fn has_uncancelled_wakeup(&self) -> Result<bool, Error> {
        let mut conn = self.db.get().await?;
        select(exists(schema::recovery_wakeup_request::table.filter(
            schema::recovery_wakeup_request::cancelled_at.is_null(),
        )))
        .get_result(&mut conn)
        .await
        .map_err(Error::from)
    }

    async fn request_wakeup(&self, operator_id: OperatorIdentityId) -> Result<(), Error> {
        let mut conn = self.db.get().await?;
        diesel::insert_into(schema::recovery_wakeup_request::table)
            .values(&NewRecoveryWakeupRequest {
                requested_by: operator_id,
            })
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    async fn cancel_wakeup(&self, operator_id: OperatorIdentityId) -> Result<bool, Error> {
        let mut conn = self.db.get().await?;
        let rows = diesel::update(schema::recovery_wakeup_request::table)
            .filter(schema::recovery_wakeup_request::cancelled_at.is_null())
            .set((
                schema::recovery_wakeup_request::cancelled_by.eq(Some(operator_id)),
                schema::recovery_wakeup_request::cancelled_at.eq(Some(SqliteTimestamp::now())),
            ))
            .execute(&mut conn)
            .await?;
        Ok(rows > 0)
    }
}
