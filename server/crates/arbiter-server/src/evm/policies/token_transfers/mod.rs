use super::{DatabaseID, EvalContext, EvalViolation};
use crate::{
    crypto::integrity::Integrable,
    db::models::{
        EvmBasicGrant, EvmTokenTransferGrant, EvmTokenTransferVolumeLimit,
        NewEvmTokenTransferGrant, NewEvmTokenTransferLog, NewEvmTokenTransferVolumeLimit,
        SqliteTimestamp,
    },
    db::schema::{
        evm_basic_grant, evm_token_transfer_grant, evm_token_transfer_log,
        evm_token_transfer_volume_limit,
    },
    evm::policies::CombinedSettings,
    evm::{
        abi::IERC20::transferCall,
        policies::{
            Grant, Policy, SharedGrantSettings, SpecificGrant, SpecificMeaning, VolumeRateLimit,
        },
        utils,
    },
};
use arbiter_tokens_registry::evm::nonfungible::{self, TokenInfo};

use alloy::{
    primitives::{Address, U256},
    sol_types::SolCall,
};
use chrono::{DateTime, Duration, Utc};
use diesel::{
    ExpressionMethods,
    dsl::{auto_type, insert_into},
    prelude::*,
    sqlite::Sqlite,
};
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::collections::HashMap;

#[auto_type]
fn grant_join() -> _ {
    evm_token_transfer_grant::table.inner_join(
        evm_basic_grant::table.on(evm_token_transfer_grant::basic_grant_id.eq(evm_basic_grant::id)),
    )
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Meaning {
    pub token: &'static TokenInfo,
    pub to: Address,
    pub value: U256,
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
impl From<Meaning> for SpecificMeaning {
    fn from(val: Meaning) -> SpecificMeaning {
        SpecificMeaning::TokenTransfer(val)
    }
}

// A grant for token transfers, which can be scoped to specific target addresses and volume limits
#[derive(Debug, Clone, arbiter_macros::Hashable)]
pub struct Settings {
    pub token_contract: Address,
    pub target: Option<Address>,
    pub volume_limits: Vec<VolumeRateLimit>,
}
impl Integrable for Settings {
    const KIND: &'static str = "TokenTransfer";
}

impl From<Settings> for SpecificGrant {
    fn from(val: Settings) -> SpecificGrant {
        SpecificGrant::TokenTransfer(val)
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
    current_transfer_value: U256,
    db: &mut impl AsyncConnection<Backend = Sqlite>,
) -> QueryResult<Vec<EvalViolation>> {
    let mut violations = Vec::new();

    let Some(longest_window) = grant
        .settings
        .specific
        .volume_limits
        .iter()
        .map(|l| l.window)
        .max()
    else {
        return Ok(violations);
    };

    let past_transfers = query_relevant_past_transfers(grant.id, longest_window, db).await?;

    for limit in &grant.settings.specific.volume_limits {
        let window_start = chrono::Utc::now() - limit.window;
        let prospective_cumulative_volume: U256 = past_transfers
            .iter()
            .filter(|(_, timestamp)| timestamp >= &window_start)
            .fold(current_transfer_value, |acc, (value, _)| acc + *value);

        if prospective_cumulative_volume > limit.max_volume {
            violations.push(EvalViolation::VolumetricLimitExceeded);
            break;
        }
    }

    Ok(violations)
}

pub struct TokenTransfer;
impl Policy for TokenTransfer {
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

        if let Some(allowed) = grant.settings.specific.target
            && allowed != meaning.to
        {
            violations.push(EvalViolation::InvalidTarget { target: meaning.to });
        }

        let rate_violations = check_volume_rate_limits(grant, meaning.value, db).await?;
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
        let token_contract_bytes = context.to.to_vec();

        let grant: Option<(EvmBasicGrant, EvmTokenTransferGrant)> = grant_join()
            .filter(evm_basic_grant::revoked_at.is_null())
            .filter(evm_basic_grant::wallet_access_id.eq(context.target.id))
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
            common_settings_id: token_grant.basic_grant_id,
            settings: CombinedSettings {
                shared: SharedGrantSettings::try_from_model(basic_grant)?,
                specific: settings,
            },
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

    async fn find_all_grants(
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> QueryResult<Vec<Grant<Self::Settings>>> {
        let grants: Vec<(EvmBasicGrant, EvmTokenTransferGrant)> = grant_join()
            .filter(evm_basic_grant::revoked_at.is_null())
            .select((
                EvmBasicGrant::as_select(),
                EvmTokenTransferGrant::as_select(),
            ))
            .load(conn)
            .await?;

        if grants.is_empty() {
            return Ok(Vec::new());
        }

        let grant_ids: Vec<i32> = grants.iter().map(|(_, g)| g.id).collect();

        let all_volume_limits: Vec<EvmTokenTransferVolumeLimit> =
            evm_token_transfer_volume_limit::table
                .filter(evm_token_transfer_volume_limit::grant_id.eq_any(&grant_ids))
                .select(EvmTokenTransferVolumeLimit::as_select())
                .load(conn)
                .await?;

        let mut limits_by_grant: HashMap<i32, Vec<EvmTokenTransferVolumeLimit>> = HashMap::new();
        for limit in all_volume_limits {
            limits_by_grant
                .entry(limit.grant_id)
                .or_default()
                .push(limit);
        }

        grants
            .into_iter()
            .map(|(basic, specific)| {
                let volume_limits: Vec<VolumeRateLimit> = limits_by_grant
                    .get(&specific.id)
                    .map(|v| v.as_slice())
                    .unwrap_or_default()
                    .iter()
                    .map(|row| {
                        Ok(VolumeRateLimit {
                            max_volume: utils::try_bytes_to_u256(&row.max_volume).map_err(|e| {
                                diesel::result::Error::DeserializationError(Box::new(e))
                            })?,
                            window: Duration::seconds(row.window_secs as i64),
                        })
                    })
                    .collect::<QueryResult<Vec<_>>>()?;

                let token_contract: [u8; 20] =
                    specific.token_contract.clone().try_into().map_err(|_| {
                        diesel::result::Error::DeserializationError(
                            "Invalid token contract address length".into(),
                        )
                    })?;

                let target: Option<Address> = match &specific.receiver {
                    None => None,
                    Some(bytes) => {
                        let arr: [u8; 20] = bytes.clone().try_into().map_err(|_| {
                            diesel::result::Error::DeserializationError(
                                "Invalid receiver address length".into(),
                            )
                        })?;
                        Some(Address::from(arr))
                    }
                };

                Ok(Grant {
                    id: specific.id,
                    common_settings_id: specific.basic_grant_id,
                    settings: CombinedSettings {
                        shared: SharedGrantSettings::try_from_model(basic)?,
                        specific: Settings {
                            token_contract: Address::from(token_contract),
                            target,
                            volume_limits,
                        },
                    },
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
