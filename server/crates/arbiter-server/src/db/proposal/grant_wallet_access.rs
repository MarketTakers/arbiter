//! Granting an SDK client visibility of a wallet.

use super::{Proposal, ProposalKindTag};
use crate::db::{DatabaseConnection, schema::proposal_grant_wallet_access as table};
use diesel::{
    ExpressionMethods as _, Insertable, QueryDsl as _, QueryResult, Queryable, Selectable,
    SelectableHelper as _, sqlite::Sqlite,
};
use diesel_async::RunQueryDsl as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Queryable, Selectable, Insertable)]
#[diesel(table_name = table, check_for_backend(Sqlite))]
pub struct Settings {
    pub wallet_id: i32,
    pub client_id: i32,
}

pub struct GrantWalletAccess;

impl Proposal for GrantWalletAccess {
    const KIND: ProposalKindTag = ProposalKindTag::GrantWalletAccess;

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
