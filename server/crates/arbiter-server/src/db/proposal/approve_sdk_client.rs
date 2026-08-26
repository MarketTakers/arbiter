//! Approving an SDK client so it may authenticate against the vault.

use super::{Proposal, ProposalKindTag};
use crate::db::{DatabaseConnection, schema::proposal_approve_sdk_client as table};
use diesel::{
    ExpressionMethods as _, Insertable, QueryDsl as _, QueryResult, Queryable, Selectable,
    SelectableHelper as _, sqlite::Sqlite,
};
use diesel_async::RunQueryDsl as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Queryable, Selectable, Insertable)]
#[diesel(table_name = table, check_for_backend(Sqlite))]
pub struct Settings {
    pub client_id: i32,
}

pub struct ApproveSdkClient;

impl Proposal for ApproveSdkClient {
    const KIND: ProposalKindTag = ProposalKindTag::ApproveSdkClient;

    type Settings = Settings;

    async fn insert(
        proposal_id: i32,
        settings: &Self::Settings,
        conn: &mut DatabaseConnection,
    ) -> QueryResult<()> {
        diesel::insert_into(table::table)
            .values((table::proposal_id.eq(proposal_id), settings))
            .execute(conn)
            .await
            .map(drop)
    }

    async fn load(proposal_id: i32, conn: &mut DatabaseConnection) -> QueryResult<Self::Settings> {
        table::table
            .find(proposal_id)
            .select(Settings::as_select())
            .first(conn)
            .await
    }
}
