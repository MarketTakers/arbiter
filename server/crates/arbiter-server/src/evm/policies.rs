use std::fmt::Display;

use alloy::primitives::{Address, Bytes, ChainId, U256};
use chrono::{DateTime, Duration, Utc};
use diesel::{
    ExpressionMethods as _, QueryDsl, SelectableHelper, result::QueryResult, sqlite::Sqlite,
};
use diesel_async::{AsyncConnection, RunQueryDsl};

use thiserror::Error;

use crate::{
    crypto::integrity::v1::Integrable,
    db::models::{self, EvmBasicGrant, EvmWalletAccess},
    evm::utils,
};

pub mod ether_transfer;
pub mod token_transfers;

#[derive(Debug, Clone)]
pub struct EvalContext {
    // Which wallet is this transaction for and who requested it
    pub target: EvmWalletAccess,

    // The transaction data
    pub chain: ChainId,
    pub to: Address,
    pub value: U256,
    pub calldata: Bytes,

    // Gas pricing (EIP-1559)
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
}

#[derive(Debug, Error)]
pub enum EvalViolation {
    #[error("This grant doesn't allow transactions to the target address {target}")]
    InvalidTarget { target: Address },

    #[error("Gas limit exceeded for this grant")]
    GasLimitExceeded {
        max_gas_fee_per_gas: Option<U256>,
        max_priority_fee_per_gas: Option<U256>,
    },

    #[error("Rate limit exceeded for this grant")]
    RateLimitExceeded,

    #[error("Transaction exceeds volumetric limits of the grant")]
    VolumetricLimitExceeded,

    #[error("Transaction is outside of the grant's validity period")]
    InvalidTime,

    #[error("Transaction type is not allowed by this grant")]
    InvalidTransactionType,

    #[error("Mismatching chain ID")]
    MismatchingChainId { expected: ChainId, actual: ChainId },
}

pub type DatabaseID = i32;

#[derive(Debug)]
pub struct Grant<PolicySettings> {
    pub id: DatabaseID,
    pub common_settings_id: DatabaseID, // ID of the basic grant for shared-logic checks like rate limits and validity periods
    pub settings: CombinedSettings<PolicySettings>,
}

pub trait Policy: Sized {
    type Settings: Send + Sync + 'static + Into<SpecificGrant> + Integrable;
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

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionRateLimit {
    pub count: u32,
    pub window: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VolumeRateLimit {
    pub max_volume: U256,
    pub window: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SharedGrantSettings {
    pub wallet_access_id: i32,
    pub chain: ChainId,

    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,

    pub max_gas_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,

    pub rate_limit: Option<TransactionRateLimit>,
}

impl SharedGrantSettings {
    pub(crate) fn try_from_model(model: EvmBasicGrant) -> QueryResult<Self> {
        Ok(Self {
            wallet_access_id: model.wallet_access_id,
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

#[derive(Debug, Clone)]
pub enum SpecificGrant {
    EtherTransfer(ether_transfer::Settings),
    TokenTransfer(token_transfers::Settings),
}

#[derive(Debug)]
pub struct CombinedSettings<PolicyGrant> {
    pub shared: SharedGrantSettings,
    pub specific: PolicyGrant,
}

impl<P> CombinedSettings<P> {
    pub fn generalize<Y: From<P>>(self) -> CombinedSettings<Y> {
        CombinedSettings {
            shared: self.shared,
            specific: self.specific.into(),
        }
    }
}

impl<P: Integrable> Integrable for CombinedSettings<P> {
    const KIND: &'static str = P::KIND;
    const VERSION: i32 = P::VERSION;
}

use crate::crypto::integrity::hashing::Hashable;

impl Hashable for TransactionRateLimit {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        self.count.hash(hasher);
        self.window.hash(hasher);
    }
}

impl Hashable for VolumeRateLimit {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        self.max_volume.hash(hasher);
        self.window.hash(hasher);
    }
}

impl Hashable for SharedGrantSettings {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        self.wallet_access_id.hash(hasher);
        self.chain.hash(hasher);
        self.valid_from.hash(hasher);
        self.valid_until.hash(hasher);
        self.max_gas_fee_per_gas.hash(hasher);
        self.max_priority_fee_per_gas.hash(hasher);
        self.rate_limit.hash(hasher);
    }
}

impl<P: Hashable> Hashable for CombinedSettings<P> {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        self.shared.hash(hasher);
        self.specific.hash(hasher);
    }
}
