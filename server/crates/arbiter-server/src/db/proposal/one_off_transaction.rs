//! Signing a single EIP-1559 transaction.

use super::{Proposal, ProposalKindTag, as_i64, as_u64, fixed};
use crate::db::{
    DatabaseConnection,
    schema::{proposal_one_off_transaction, proposal_one_off_transaction_result},
};
use diesel::{
    Insertable, QueryDsl as _, QueryResult, Queryable, Selectable, SelectableHelper as _,
    sqlite::Sqlite,
};
use diesel_async::RunQueryDsl as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub client_id: i32,
    pub wallet_address: [u8; 20],
    pub chain_id: u64,
    pub nonce: u64,
    pub gas_limit: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub to: [u8; 20],
    pub value: [u8; 32],
    pub input: Vec<u8>,
}

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = proposal_one_off_transaction, check_for_backend(Sqlite))]
struct Row {
    proposal_id: i32,
    client_id: i32,
    wallet_address: Vec<u8>,
    chain_id: i64,
    nonce: i64,
    gas_limit: i64,
    max_fee_per_gas: Vec<u8>,
    max_priority_fee_per_gas: Vec<u8>,
    to_address: Vec<u8>,
    value: Vec<u8>,
    input: Vec<u8>,
}

impl Row {
    fn new(proposal_id: i32, settings: &Settings) -> QueryResult<Self> {
        Ok(Self {
            proposal_id,
            client_id: settings.client_id,
            wallet_address: settings.wallet_address.to_vec(),
            chain_id: as_i64(settings.chain_id)?,
            nonce: as_i64(settings.nonce)?,
            gas_limit: as_i64(settings.gas_limit)?,
            max_fee_per_gas: settings.max_fee_per_gas.to_be_bytes().to_vec(),
            max_priority_fee_per_gas: settings.max_priority_fee_per_gas.to_be_bytes().to_vec(),
            to_address: settings.to.to_vec(),
            value: settings.value.to_vec(),
            input: settings.input.clone(),
        })
    }

    fn into_settings(self) -> QueryResult<Settings> {
        Ok(Settings {
            client_id: self.client_id,
            wallet_address: fixed!(self.wallet_address)?,
            chain_id: as_u64(self.chain_id)?,
            nonce: as_u64(self.nonce)?,
            gas_limit: as_u64(self.gas_limit)?,
            max_fee_per_gas: u128::from_be_bytes(fixed!(self.max_fee_per_gas)?),
            max_priority_fee_per_gas: u128::from_be_bytes(fixed!(self.max_priority_fee_per_gas)?),
            to: fixed!(self.to_address)?,
            value: fixed!(self.value)?,
            input: self.input,
        })
    }
}

pub struct OneOffTransaction;

impl Proposal for OneOffTransaction {
    const KIND: ProposalKindTag = ProposalKindTag::ApproveOneOffTransaction;

    type Settings = Settings;

    async fn insert(
        proposal_id: i32,
        settings: &Self::Settings,
        conn: &mut DatabaseConnection,
    ) -> QueryResult<()> {
        diesel::insert_into(proposal_one_off_transaction::table)
            .values(&Row::new(proposal_id, settings)?)
            .execute(conn)
            .await
            .map(drop)
    }

    async fn load(proposal_id: i32, conn: &mut DatabaseConnection) -> QueryResult<Self::Settings> {
        let row: Row = proposal_one_off_transaction::table
            .find(proposal_id)
            .select(Row::as_select())
            .first(conn)
            .await?;

        row.into_settings()
    }
}

/// The signature the vault produced for an approved transaction.
#[derive(Debug, Insertable)]
#[diesel(table_name = proposal_one_off_transaction_result, check_for_backend(Sqlite))]
struct SignatureRow {
    proposal_id: i32,
    r: Vec<u8>,
    s: Vec<u8>,
    y_parity: i32,
}

/// Records the signature produced for an approved transaction, by component, so what
/// came back is as readable as what was signed.
pub async fn store_signature(
    proposal_id: i32,
    signature: &alloy::signers::Signature,
    conn: &mut DatabaseConnection,
) -> QueryResult<()> {
    diesel::insert_into(proposal_one_off_transaction_result::table)
        .values(&SignatureRow {
            proposal_id,
            r: signature.r().to_be_bytes::<32>().to_vec(),
            s: signature.s().to_be_bytes::<32>().to_vec(),
            y_parity: i32::from(signature.v()),
        })
        .execute(conn)
        .await
        .map(drop)
}
