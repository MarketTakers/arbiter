use std::fmt::Display;

use alloy::primitives::{Address, U256};
use chrono::{DateTime, Duration, Utc};
use diesel::dsl::insert_into;
use diesel::sqlite::Sqlite;
use diesel::{ExpressionMethods, JoinOnDsl, prelude::*};
use diesel_async::{AsyncConnection, RunQueryDsl};

use crate::db::models::{
    EvmBasicGrant, EvmEtherTransferGrant, EvmEtherTransferGrantTarget, EvmEtherTransferLimit,
    NewEvmEtherTransferLimit, SqliteTimestamp,
};
use crate::db::schema::{evm_ether_transfer_limit, evm_transaction_log};
use crate::evm::policies::{
    Grant, SharedGrantSettings, SpecificGrant, SpecificMeaning, VolumeRateLimit,
};
use crate::{
    db::{
        models::{self, NewEvmEtherTransferGrant, NewEvmEtherTransferGrantTarget},
        schema::{evm_ether_transfer_grant, evm_ether_transfer_grant_target},
    },
    evm::{policies::Policy, utils},
};

use super::{DatabaseID, EvalContext, EvalViolation};

// Plain ether transfer
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Meaning {
    to: Address,
    value: U256,
}
impl Display for Meaning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ether transfer of {} to {}",
            self.value,
            self.to.to_string()
        )
    }
}
impl Into<SpecificMeaning> for Meaning {
    fn into(self) -> SpecificMeaning {
        SpecificMeaning::EtherTransfer(self)
    }
}

// A grant for ether transfers, which can be scoped to specific target addresses and volume limits
pub struct Settings {
    target: Vec<Address>,
    limit: VolumeRateLimit,
}

impl Into<SpecificGrant> for Settings {
    fn into(self) -> SpecificGrant {
        SpecificGrant::EtherTransfer(self)
    }
}

async fn query_relevant_past_transaction(
    grant_id: i32,
    longest_window: Duration,
    db: &mut impl AsyncConnection<Backend = Sqlite>,
) -> QueryResult<Vec<(U256, DateTime<Utc>)>> {
    let past_transactions: Vec<(Vec<u8>, SqliteTimestamp)> = evm_transaction_log::table
        .filter(evm_transaction_log::grant_id.eq(grant_id))
        .filter(
            evm_transaction_log::signed_at.ge(SqliteTimestamp(chrono::Utc::now() - longest_window)),
        )
        .select((
            evm_transaction_log::eth_value,
            evm_transaction_log::signed_at,
        ))
        .load(db)
        .await?;
    let past_transaction: Vec<(U256, DateTime<Utc>)> = past_transactions
        .into_iter()
        .filter_map(|(value_bytes, timestamp)| {
            let value = utils::bytes_to_u256(&value_bytes)?;
            Some((value, timestamp.0))
        })
        .collect();
    Ok(past_transaction)
}

async fn check_rate_limits(
    grant: &Grant<Settings>,
    db: &mut impl AsyncConnection<Backend = Sqlite>,
) -> QueryResult<Vec<EvalViolation>> {
    let mut violations = Vec::new();
    let window = grant.settings.limit.window;

    let past_transaction = query_relevant_past_transaction(grant.id, window, db).await?;

    let window_start = chrono::Utc::now() - grant.settings.limit.window;
    let cumulative_volume: U256 = past_transaction
        .iter()
        .filter(|(_, timestamp)| timestamp >= &window_start)
        .fold(U256::default(), |acc, (value, _)| acc + *value);

    if cumulative_volume > grant.settings.limit.max_volume {
        violations.push(EvalViolation::VolumetricLimitExceeded);
    }

    Ok(violations)
}

pub struct EtherTransfer;
impl Policy for EtherTransfer {
    type Settings = Settings;

    type Meaning = Meaning;

    fn analyze(context: &EvalContext) -> Option<Self::Meaning> {
        if !context.calldata.is_empty() {
            return None;
        }

        Some(Meaning {
            to: context.to,
            value: context.value,
        })
    }

    async fn evaluate(
        _: &EvalContext,
        meaning: &Self::Meaning,
        grant: &Grant<Self::Settings>,
        db: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> QueryResult<Vec<EvalViolation>> {
        let mut violations = Vec::new();

        // Check if the target address is within the grant's allowed targets
        if !grant.settings.target.contains(&meaning.to) {
            violations.push(EvalViolation::InvalidTarget { target: meaning.to });
        }

        let rate_violations = check_rate_limits(grant, db).await?;
        violations.extend(rate_violations);

        Ok(violations)
    }

    async fn create_grant(
        basic: &models::EvmBasicGrant,
        grant: &Self::Settings,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> diesel::result::QueryResult<DatabaseID> {
        let limit_id: i32 = insert_into(evm_ether_transfer_limit::table)
            .values(NewEvmEtherTransferLimit {
                window_secs: grant.limit.window.num_seconds() as i32,
                max_volume: utils::u256_to_bytes(grant.limit.max_volume).to_vec(),
            })
            .returning(evm_ether_transfer_limit::id)
            .get_result(conn)
            .await?;

        let grant_id: i32 = insert_into(evm_ether_transfer_grant::table)
            .values(&NewEvmEtherTransferGrant {
                basic_grant_id: basic.id,
                limit_id,
            })
            .returning(evm_ether_transfer_grant::id)
            .get_result(conn)
            .await?;

        for target in &grant.target {
            insert_into(evm_ether_transfer_grant_target::table)
                .values(NewEvmEtherTransferGrantTarget {
                    grant_id,
                    address: target.to_vec(),
                })
                .execute(conn)
                .await?;
        }

        Ok(grant_id)
    }

    async fn try_find_grant(
        context: &EvalContext,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> diesel::result::QueryResult<Option<Grant<Self::Settings>>> {
        use crate::db::schema::{
            evm_basic_grant, evm_ether_transfer_grant, evm_ether_transfer_grant_target,
        };

        let target_bytes = context.to.to_vec();

        // Find a grant where:
        // 1. The basic grant's wallet_id and client_id match the context
        // 2. Any of the grant's targets match the context's `to` address
        let grant: Option<(EvmBasicGrant, EvmEtherTransferGrant)> = evm_ether_transfer_grant::table
            .inner_join(
                evm_basic_grant::table
                    .on(evm_ether_transfer_grant::basic_grant_id.eq(evm_basic_grant::id)),
            )
            .inner_join(
                evm_ether_transfer_grant_target::table
                    .on(evm_ether_transfer_grant::id.eq(evm_ether_transfer_grant_target::grant_id)),
            )
            .filter(evm_basic_grant::wallet_id.eq(context.wallet_id))
            .filter(evm_basic_grant::client_id.eq(context.client_id))
            .filter(evm_ether_transfer_grant_target::address.eq(&target_bytes))
            .select((
                EvmBasicGrant::as_select(),
                EvmEtherTransferGrant::as_select(),
            ))
            .first(conn)
            .await
            .optional()?;

        let Some((basic_grant, grant)) = grant else {
            return Ok(None);
        };

        let target_bytes: Vec<EvmEtherTransferGrantTarget> = evm_ether_transfer_grant_target::table
            .select(EvmEtherTransferGrantTarget::as_select())
            .filter(evm_ether_transfer_grant_target::grant_id.eq(grant.id))
            .load(conn)
            .await?;

        let limit: EvmEtherTransferLimit = evm_ether_transfer_limit::table
            .filter(evm_ether_transfer_limit::id.eq(grant.limit_id))
            .select(EvmEtherTransferLimit::as_select())
            .first::<EvmEtherTransferLimit>(conn)
            .await?;

        // Convert bytes back to Address
        let targets: Vec<Address> = target_bytes
            .into_iter()
            .filter_map(|target| {
                // TODO: Handle invalid addresses more gracefully
                let arr: [u8; 20] = target.address.try_into().ok()?;
                Some(Address::from(arr))
            })
            .collect();

        let settings = Settings {
            target: targets,
            limit: VolumeRateLimit {
                max_volume: utils::try_bytes_to_u256(&limit.max_volume)
                    .map_err(|err| diesel::result::Error::DeserializationError(Box::new(err)))?,
                window: chrono::Duration::seconds(limit.window_secs as i64),
            },
        };

        Ok(Some(Grant {
            id: grant.id,
            shared_grant_id: grant.basic_grant_id,
            shared: SharedGrantSettings::try_from_model(basic_grant)?,
            settings,
        }))
    }

    async fn record_transaction(
        _context: &EvalContext,
        _: &Self::Meaning,
        _log_id: i32,
        _grant: &Grant<Self::Settings>,
        _conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> diesel::result::QueryResult<()> {
        // Basic log is sufficient

        Ok(())
    }
}
