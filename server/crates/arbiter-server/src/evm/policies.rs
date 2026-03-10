use std::fmt::Display;

use alloy::primitives::{Address, Bytes, ChainId, U256};
use chrono::{DateTime, Duration, Utc};
use diesel::{
    ExpressionMethods as _, QueryDsl, SelectableHelper, result::QueryResult, sqlite::Sqlite,
};
use diesel_async::{AsyncConnection, RunQueryDsl};
use miette::Diagnostic;
use thiserror::Error;

use crate::{
    db::models::{self, EvmBasicGrant},
    evm::utils,
};

pub mod ether_transfer;
pub mod token_transfers;

pub struct EvalContext {
    // Which wallet is this transaction for
    pub client_id: i32,
    pub wallet_id: i32,

    // The transaction data
    pub chain: ChainId,
    pub to: Address,
    pub value: U256,
    pub calldata: Bytes,

    // Gas pricing (EIP-1559)
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
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

    #[error("Transaction type is not allowed by this grant")]
    #[diagnostic(code(arbiter_server::evm::eval_violation::invalid_transaction_type))]
    InvalidTransactionType,
}

pub type DatabaseID = i32;

pub struct Grant<PolicySettings> {
    pub id: DatabaseID,
    pub shared_grant_id: DatabaseID, // ID of the basic grant for shared-logic checks like rate limits and validity periods
    pub shared: SharedGrantSettings,
    pub settings: PolicySettings,
}

pub trait Policy: Sized {
    type Settings: Send + Sync + 'static + Into<SpecificGrant>;
    type Meaning: Display + std::fmt::Debug + Send + Sync + 'static + Into<SpecificMeaning>;

    fn analyze(context: &EvalContext) -> Option<Self::Meaning>;

    // Evaluate whether a transaction with the given meaning complies with the provided grant, and return any violations if not
    // Empty vector means transaction is compliant with the grant
    fn evaluate(
        context: &EvalContext,
        meaning: &Self::Meaning,
        grant: &Grant<Self::Settings>,
        db: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> impl Future<Output = QueryResult<Vec<EvalViolation>>> + Send;

    // Create a new grant in the database based on the provided grant details, and return its ID
    fn create_grant(
        basic: &models::EvmBasicGrant,
        grant: &Self::Settings,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> impl std::future::Future<Output = QueryResult<DatabaseID>> + Send;

    // Try to find an existing grant that matches the transaction context, and return its details if found
    // Additionally, return ID of basic grant for shared-logic checks like rate limits and validity periods
    fn try_find_grant(
        context: &EvalContext,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> impl Future<Output = QueryResult<Option<Grant<Self::Settings>>>> + Send;

    // Return all non-revoked grants, eagerly loading policy-specific settings
    fn find_all_grants(
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> impl Future<Output = QueryResult<Vec<Grant<Self::Settings>>>> + Send;

    // Records, updates or deletes rate limits
    // In other words, records grant-specific things after transaction is executed
    fn record_transaction(
        context: &EvalContext,
        meaning: &Self::Meaning,
        log_id: i32,
        grant: &Grant<Self::Settings>,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> impl Future<Output = QueryResult<()>> + Send;
}

pub enum ReceiverTarget {
    Specific(Vec<Address>), // only allow transfers to these addresses
    Any,                    // allow transfers to any address
}

// Classification of what transaction does
#[derive(Debug)]
pub enum SpecificMeaning {
    EtherTransfer(ether_transfer::Meaning),
    TokenTransfer(token_transfers::Meaning),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TransactionRateLimit {
    pub count: u32,
    pub window: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VolumeRateLimit {
    pub max_volume: U256,
    pub window: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SharedGrantSettings {
    pub wallet_id: i32,
    pub chain: ChainId,

    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,

    pub max_gas_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,

    pub rate_limit: Option<TransactionRateLimit>,
}

impl SharedGrantSettings {
    fn try_from_model(model: EvmBasicGrant) -> QueryResult<Self> {
        Ok(Self {
            wallet_id: model.wallet_id,
            chain: model.chain_id as u64, // safe because chain_id is stored as i32 but is guaranteed to be a valid ChainId by the API when creating grants
            valid_from: model.valid_from.map(Into::into),
            valid_until: model.valid_until.map(Into::into),
            max_gas_fee_per_gas: model
                .max_gas_fee_per_gas
                .map(|b| utils::try_bytes_to_u256(&b))
                .transpose()?,
            max_priority_fee_per_gas: model
                .max_priority_fee_per_gas
                .map(|b| utils::try_bytes_to_u256(&b))
                .transpose()?,
            rate_limit: match (model.rate_limit_count, model.rate_limit_window_secs) {
                (Some(count), Some(window_secs)) => Some(TransactionRateLimit {
                    count: count as u32,
                    window: Duration::seconds(window_secs as i64),
                }),
                _ => None,
            },
        })
    }

    pub async fn query_by_id(
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
        id: i32,
    ) -> diesel::result::QueryResult<Self> {
        use crate::db::schema::evm_basic_grant;

        let basic_grant: EvmBasicGrant = evm_basic_grant::table
            .select(EvmBasicGrant::as_select())
            .filter(evm_basic_grant::id.eq(id))
            .first::<EvmBasicGrant>(conn)
            .await?;

        Self::try_from_model(basic_grant)
    }
}

pub enum SpecificGrant {
    EtherTransfer(ether_transfer::Settings),
    TokenTransfer(token_transfers::Settings),
}

/// Blanket conversion from a typed `Grant<S>` into `Grant<SpecificGrant>`.
/// Lets the engine collect across all policies into one `Vec<Grant<SpecificGrant>>`.
impl<S: Into<SpecificGrant>> From<Grant<S>> for Grant<SpecificGrant> {
    fn from(g: Grant<S>) -> Self {
        Grant {
            id: g.id,
            shared_grant_id: g.shared_grant_id,
            shared: g.shared,
            settings: g.settings.into(),
        }
    }
}

pub struct FullGrant<PolicyGrant> {
    pub basic: SharedGrantSettings,
    pub specific: PolicyGrant,
}
