//! Replacing an operator's key, which also triggers a Shamir re-key (§3.3).

use super::{Proposal, ProposalKindTag};
use crate::db::{
    DatabaseConnection,
    models::{OperatorIdentityId, ProposalId},
    schema::proposal_replace_operator as table,
};
use diesel::{
    ExpressionMethods as _, Insertable, QueryDsl as _, QueryResult, Queryable, Selectable,
    SelectableHelper as _, sqlite::Sqlite,
};
use diesel_async::RunQueryDsl as _;

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable)]
#[diesel(table_name = table, check_for_backend(Sqlite))]
pub struct Settings {
    pub old_operator_id: OperatorIdentityId,
    pub new_pubkey: Vec<u8>,
}

pub struct ReplaceOperator;

impl Proposal for ReplaceOperator {
    const KIND: ProposalKindTag = ProposalKindTag::ReplaceOperator;

    type Settings = Settings;

    async fn insert(
        proposal_id: ProposalId,
        settings: &Self::Settings,
        conn: &mut DatabaseConnection,
    ) -> QueryResult<()> {
        diesel::insert_into(table::table)
            .values((table::proposal_id.eq(proposal_id), settings))
            .execute(conn)
            .await
            .map(drop)
    }

    async fn load(
        proposal_id: ProposalId,
        conn: &mut DatabaseConnection,
    ) -> QueryResult<Self::Settings> {
        table::table
            .find(proposal_id)
            .select(Settings::as_select())
            .first(conn)
            .await
    }
}
