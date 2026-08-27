//! A Shamir re-key over the current operator set (§3.3).

use super::{Proposal, ProposalKindTag};
use crate::db::{DatabaseConnection, models::ProposalId};
use diesel::QueryResult;

pub struct TriggerRekey;

impl Proposal for TriggerRekey {
    const KIND: ProposalKindTag = ProposalKindTag::TriggerRekey;

    type Settings = ();

    async fn insert(
        _proposal_id: ProposalId,
        _settings: &Self::Settings,
        _conn: &mut DatabaseConnection,
    ) -> QueryResult<()> {
        Ok(())
    }

    async fn load(
        _proposal_id: ProposalId,
        _conn: &mut DatabaseConnection,
    ) -> QueryResult<Self::Settings> {
        Ok(())
    }
}
