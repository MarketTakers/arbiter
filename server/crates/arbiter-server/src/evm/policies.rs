use std::fmt::Display;

use alloy::primitives::{Address, Bytes, ChainId, U256};
use chrono::{DateTime, Duration, Utc};
use diesel::{result::QueryResult, sqlite::Sqlite};
use diesel_async::AsyncConnection;
use miette::Diagnostic;
use thiserror::Error;

use crate::db::models;

pub mod ether_transfer;

pub struct EvalContext {
    // Which wallet is this transaction for
    pub client_id: i32,
    pub wallet_id: i32,

    // The transaction data
    pub chain: ChainId,
    pub to: Address,
    pub value: U256,
    pub calldata: Bytes,
}

#[derive(Debug, Error, Diagnostic)]
pub enum EvalViolation {
    #[error("This grant doesn't allow transactions to the target address {target}")]
    #[diagnostic(code(arbiter_server::evm::eval_violation::invalid_target))]
    InvalidTarget { target: Address },

    #[error("Gas limit exceeded for this grant")]
    #[diagnostic(code(arbiter_server::evm::eval_violation::gas_limit_exceeded))]
    GasLimitExceeded {
        max_gas_fee_per_gas: Option<U256>,
        max_priority_fee_per_gas: Option<U256>,
    },

    #[error("Rate limit exceeded for this grant")]
    #[diagnostic(code(arbiter_server::evm::eval_violation::rate_limit_exceeded))]
    RateLimitExceeded,

    #[error("Transaction exceeds volumetric limits of the grant")]
    #[diagnostic(code(arbiter_server::evm::eval_violation::volumetric_limit_exceeded))]
    VolumetricLimitExceeded,

    #[error("Transaction is outside of the grant's validity period")]
    #[diagnostic(code(arbiter_server::evm::eval_violation::invalid_time))]
    InvalidTime,
}

pub type DatabaseID = i32;

pub struct GrantMetadata {
    pub basic_grant_id: DatabaseID,
    pub policy_grant_id: DatabaseID,
}

pub trait Policy: Sized {
    type Grant: Send + 'static + Into<SpecificGrant>;
    type Meaning: Display + Send + 'static + Into<SpecificMeaning>;

    fn analyze(context: &EvalContext) -> Option<Self::Meaning>;

    // Evaluate whether a transaction with the given meaning complies with the provided grant, and return any violations if not
    // Empty vector means transaction is compliant with the grant
    fn evaluate(
        meaning: &Self::Meaning,
        grant: &Self::Grant,
        meta: &GrantMetadata,
        db: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> impl Future<Output = QueryResult<Vec<EvalViolation>>> + Send;

    // Create a new grant in the database based on the provided grant details, and return its ID
    fn create_grant(
        basic: &models::EvmBasicGrant,
        grant: &Self::Grant,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> impl std::future::Future<Output = QueryResult<DatabaseID>> + Send;

    // Try to find an existing grant that matches the transaction context, and return its details if found
    // Additionally, return ID of basic grant for shared-logic checks like rate limits and validity periods
    fn try_find_grant(
        context: &EvalContext,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> impl Future<Output = QueryResult<Option<(Self::Grant, GrantMetadata)>>>;

    // Records, updates or deletes rate limits
    // In other words, records grant-specific things after transaction is executed
    fn record_transaction(
        context: &EvalContext,
        grant: &GrantMetadata,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> impl Future<Output = QueryResult<()>>;
}

// Classification of what transaction does
pub enum SpecificMeaning {
    EtherTransfer(ether_transfer::Meaning),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TransactionRateLimit {
    pub count: u32,
    pub window: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BasicGrant {
    pub wallet_id: i32,
    pub chain: ChainId,

    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,

    pub max_gas_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,

    pub rate_limit: Option<TransactionRateLimit>,
}

pub enum SpecificGrant {
    EtherTransfer(ether_transfer::Grant),
}

pub struct FullGrant<PolicyGrant> {
    pub basic: BasicGrant,
    pub specific: PolicyGrant,
}