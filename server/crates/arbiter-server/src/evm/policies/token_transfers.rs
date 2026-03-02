use alloy::{
    primitives::{Address, U256},
    sol_types::SolCall,
};
use arbiter_tokens_registry::evm::nonfungible::{self, TokenInfo};
use chrono::{DateTime, Duration, Utc};
use diesel::dsl::insert_into;
use diesel::sqlite::Sqlite;
use diesel::{ExpressionMethods, prelude::*};
use diesel_async::{AsyncConnection, RunQueryDsl};

use crate::db::models::{
    EvmBasicGrant, EvmTokenTransferGrant, EvmTokenTransferVolumeLimit, NewEvmTokenTransferGrant,
    NewEvmTokenTransferLog, NewEvmTokenTransferVolumeLimit, SqliteTimestamp,
};
use crate::db::schema::{
    evm_token_transfer_grant, evm_token_transfer_log, evm_token_transfer_volume_limit,
};
use crate::evm::{
    abi::IERC20::transferCall,
    policies::{Grant, Policy, SharedGrantSettings, SpecificGrant, SpecificMeaning, VolumeRateLimit},
    utils,
};

use super::{DatabaseID, EvalContext, EvalViolation};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Meaning {
    token: &'static TokenInfo,
    to: Address,
    value: U256,
}
impl std::fmt::Display for Meaning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transfer of {} {} to {}",
            self.value, self.token.symbol, self.to
        )
    }
}
impl Into<SpecificMeaning> for Meaning {
    fn into(self) -> SpecificMeaning {
        SpecificMeaning::TokenTransfer(self)
    }
}

// A grant for token transfers, which can be scoped to specific target addresses and volume limits
pub struct Settings {
    token_contract: Address,
    target: Option<Address>,
    volume_limits: Vec<VolumeRateLimit>,
}
impl Into<SpecificGrant> for Settings {
    fn into(self) -> SpecificGrant {
        SpecificGrant::TokenTransfer(self)
    }
}

async fn query_relevant_past_transfers(
    grant_id: i32,
    longest_window: Duration,
    db: &mut impl AsyncConnection<Backend = Sqlite>,
) -> QueryResult<Vec<(U256, DateTime<Utc>)>> {
    let past_logs: Vec<(Vec<u8>, SqliteTimestamp)> = evm_token_transfer_log::table
        .filter(evm_token_transfer_log::grant_id.eq(grant_id))
        .filter(
            evm_token_transfer_log::created_at
                .ge(SqliteTimestamp(chrono::Utc::now() - longest_window)),
        )
        .select((
            evm_token_transfer_log::value,
            evm_token_transfer_log::created_at,
        ))
        .load(db)
        .await?;

    let past_transfers: Vec<(U256, DateTime<Utc>)> = past_logs
        .into_iter()
        .filter_map(|(value_bytes, timestamp)| {
            let value = utils::bytes_to_u256(&value_bytes)?;
            Some((value, timestamp.0))
        })
        .collect();

    Ok(past_transfers)
}

async fn check_volume_rate_limits(
    grant: &Grant<Settings>,
    db: &mut impl AsyncConnection<Backend = Sqlite>,
) -> QueryResult<Vec<EvalViolation>> {
    let mut violations = Vec::new();

    let Some(longest_window) = grant.settings.volume_limits.iter().map(|l| l.window).max() else {
        return Ok(violations);
    };

    let past_transfers = query_relevant_past_transfers(grant.id, longest_window, db).await?;

    for limit in &grant.settings.volume_limits {
        let window_start = chrono::Utc::now() - limit.window;
        let cumulative_volume: U256 = past_transfers
            .iter()
            .filter(|(_, timestamp)| timestamp >= &window_start)
            .fold(U256::default(), |acc, (value, _)| acc + *value);

        if cumulative_volume > limit.max_volume {
            violations.push(EvalViolation::VolumetricLimitExceeded);
            break;
        }
    }

    Ok(violations)
}

pub struct TokenTransferPolicy;
impl Policy for TokenTransferPolicy {
    type Settings = Settings;
    type Meaning = Meaning;

    fn analyze(context: &EvalContext) -> Option<Self::Meaning> {
        let token = nonfungible::get_token(context.chain, context.to)?;
        let decoded = transferCall::abi_decode_raw_validate(&context.calldata).ok()?;

        Some(Meaning {
            token,
            to: decoded.to,
            value: decoded.value,
        })
    }

    async fn evaluate(
        context: &EvalContext,
        meaning: &Self::Meaning,
        grant: &Grant<Self::Settings>,
        db: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> QueryResult<Vec<EvalViolation>> {
        let mut violations = Vec::new();

        // erc20 transfer shouldn't carry eth value
        if !context.value.is_zero() {
            violations.push(EvalViolation::InvalidTransactionType);
            return Ok(violations);
        }

        if let Some(allowed) = grant.settings.target {
            if allowed != meaning.to {
                violations.push(EvalViolation::InvalidTarget { target: meaning.to });
            }
        }

        let rate_violations = check_volume_rate_limits(grant, db).await?;
        violations.extend(rate_violations);

        Ok(violations)
    }

    async fn create_grant(
        basic: &EvmBasicGrant,
        grant: &Self::Settings,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> QueryResult<DatabaseID> {
        // Store the specific receiver as bytes (None means any receiver is allowed)
        let receiver: Option<Vec<u8>> = grant.target.map(|addr| addr.to_vec());

        let grant_id: i32 = insert_into(evm_token_transfer_grant::table)
            .values(NewEvmTokenTransferGrant {
                basic_grant_id: basic.id,
                token_contract: grant.token_contract.to_vec(),
                receiver,
            })
            .returning(evm_token_transfer_grant::id)
            .get_result(conn)
            .await?;

        for limit in &grant.volume_limits {
            insert_into(evm_token_transfer_volume_limit::table)
                .values(NewEvmTokenTransferVolumeLimit {
                    grant_id,
                    window_secs: limit.window.num_seconds() as i32,
                    max_volume: utils::u256_to_bytes(limit.max_volume).to_vec(),
                })
                .execute(conn)
                .await?;
        }

        Ok(grant_id)
    }

    async fn try_find_grant(
        context: &EvalContext,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> QueryResult<Option<Grant<Self::Settings>>> {
        use crate::db::schema::{evm_basic_grant, evm_token_transfer_grant};

        let token_contract_bytes = context.to.to_vec();

        let grant: Option<(EvmBasicGrant, EvmTokenTransferGrant)> = evm_token_transfer_grant::table
            .inner_join(
                evm_basic_grant::table
                    .on(evm_token_transfer_grant::basic_grant_id.eq(evm_basic_grant::id)),
            )
            .filter(evm_basic_grant::wallet_id.eq(context.wallet_id))
            .filter(evm_basic_grant::client_id.eq(context.client_id))
            .filter(evm_token_transfer_grant::token_contract.eq(&token_contract_bytes))
            .select((
                EvmBasicGrant::as_select(),
                EvmTokenTransferGrant::as_select(),
            ))
            .first(conn)
            .await
            .optional()?;

        let Some((basic_grant, token_grant)) = grant else {
            return Ok(None);
        };

        let volume_limits_db: Vec<EvmTokenTransferVolumeLimit> =
            evm_token_transfer_volume_limit::table
                .filter(evm_token_transfer_volume_limit::grant_id.eq(token_grant.id))
                .select(EvmTokenTransferVolumeLimit::as_select())
                .load(conn)
                .await?;

        let volume_limits: Vec<VolumeRateLimit> = volume_limits_db
            .into_iter()
            .map(|row| {
                Ok(VolumeRateLimit {
                    max_volume: utils::try_bytes_to_u256(&row.max_volume).map_err(|err| {
                        diesel::result::Error::DeserializationError(Box::new(err))
                    })?,
                    window: Duration::seconds(row.window_secs as i64),
                })
            })
            .collect::<QueryResult<Vec<_>>>()?;

        let token_contract: [u8; 20] = token_grant.token_contract.try_into().map_err(|_| {
            diesel::result::Error::DeserializationError(
                "Invalid token contract address length".into(),
            )
        })?;

        let target: Option<Address> = match token_grant.receiver {
            None => None,
            Some(bytes) => {
                let arr: [u8; 20] = bytes.try_into().map_err(|_| {
                    diesel::result::Error::DeserializationError(
                        "Invalid receiver address length".into(),
                    )
                })?;
                Some(Address::from(arr))
            }
        };

        let settings = Settings {
            token_contract: Address::from(token_contract),
            target,
            volume_limits,
        };

        Ok(Some(Grant {
            id: token_grant.id,
            shared_grant_id: token_grant.basic_grant_id,
            shared: SharedGrantSettings::try_from_model(basic_grant)?,
            settings,
        }))
    }

    async fn record_transaction(
        context: &EvalContext,
        meaning: &Self::Meaning,
        log_id: i32,
        grant: &Grant<Self::Settings>,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> QueryResult<()> {
        insert_into(evm_token_transfer_log::table)
            .values(NewEvmTokenTransferLog {
                grant_id: grant.id,
                log_id,
                chain_id: context.chain as i32,
                token_contract: context.to.to_vec(),
                recipient_address: meaning.to.to_vec(),
                value: utils::u256_to_bytes(meaning.value).to_vec(),
            })
            .execute(conn)
            .await?;

        Ok(())
    }
}
