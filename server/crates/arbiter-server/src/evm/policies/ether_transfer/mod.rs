use std::collections::HashMap;
use std::fmt::Display;

use alloy::primitives::{Address, U256};
use chrono::{DateTime, Duration, Utc};
use diesel::dsl::{auto_type, insert_into};
use diesel::sqlite::Sqlite;
use diesel::{ExpressionMethods, JoinOnDsl, prelude::*};
use diesel_async::{AsyncConnection, RunQueryDsl};

use crate::db::models::{
    EvmBasicGrant, EvmEtherTransferGrant, EvmEtherTransferGrantTarget, EvmEtherTransferLimit,
    NewEvmEtherTransferLimit, SqliteTimestamp,
};
use crate::db::schema::{evm_basic_grant, evm_ether_transfer_limit, evm_transaction_log};
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

#[auto_type]
fn grant_join() -> _ {
    evm_ether_transfer_grant::table.inner_join(
        evm_basic_grant::table.on(evm_ether_transfer_grant::basic_grant_id.eq(evm_basic_grant::id)),
    )
}

use super::{DatabaseID, EvalContext, EvalViolation};

// Plain ether transfer
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Meaning {
    pub(crate) to: Address,
    pub(crate) value: U256,
}
impl Display for Meaning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ether transfer of {} to {}", self.value, self.to)
    }
}
impl From<Meaning> for SpecificMeaning {
    fn from(val: Meaning) -> SpecificMeaning {
        SpecificMeaning::EtherTransfer(val)
    }
}

// A grant for ether transfers, which can be scoped to specific target addresses and volume limits
#[derive(Debug, Clone)]
pub struct Settings {
    pub target: Vec<Address>,
    pub limit: VolumeRateLimit,
}

impl From<Settings> for SpecificGrant {
    fn from(val: Settings) -> SpecificGrant {
        SpecificGrant::EtherTransfer(val)
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
        let target_bytes = context.to.to_vec();

        // Find a grant where:
        // 1. The basic grant's wallet_id and client_id match the context
        // 2. Any of the grant's targets match the context's `to` address
        let grant: Option<(EvmBasicGrant, EvmEtherTransferGrant)> = evm_ether_transfer_grant::table
            .inner_join(evm_basic_grant::table)
            .inner_join(evm_ether_transfer_grant_target::table)
            .filter(
                evm_basic_grant::wallet_id
                    .eq(context.wallet_id)
                    .and(evm_basic_grant::client_id.eq(context.client_id))
                    .and(evm_basic_grant::revoked_at.is_null())
                    .and(evm_ether_transfer_grant_target::address.eq(&target_bytes)),
            )
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

    async fn find_all_grants(
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> QueryResult<Vec<Grant<Self::Settings>>> {
        let grants: Vec<(EvmBasicGrant, EvmEtherTransferGrant)> = grant_join()
            .filter(evm_basic_grant::revoked_at.is_null())
            .select((
                EvmBasicGrant::as_select(),
                EvmEtherTransferGrant::as_select(),
            ))
            .load(conn)
            .await?;

        if grants.is_empty() {
            return Ok(Vec::new());
        }

        let grant_ids: Vec<i32> = grants.iter().map(|(_, g)| g.id).collect();
        let limit_ids: Vec<i32> = grants.iter().map(|(_, g)| g.limit_id).collect();

        let all_targets: Vec<EvmEtherTransferGrantTarget> = evm_ether_transfer_grant_target::table
            .filter(evm_ether_transfer_grant_target::grant_id.eq_any(&grant_ids))
            .select(EvmEtherTransferGrantTarget::as_select())
            .load(conn)
            .await?;

        let all_limits: Vec<EvmEtherTransferLimit> = evm_ether_transfer_limit::table
            .filter(evm_ether_transfer_limit::id.eq_any(&limit_ids))
            .select(EvmEtherTransferLimit::as_select())
            .load(conn)
            .await?;

        let mut targets_by_grant: HashMap<i32, Vec<EvmEtherTransferGrantTarget>> = HashMap::new();
        for target in all_targets {
            targets_by_grant
                .entry(target.grant_id)
                .or_default()
                .push(target);
        }

        let limits_by_id: HashMap<i32, EvmEtherTransferLimit> =
            all_limits.into_iter().map(|l| (l.id, l)).collect();

        grants
            .into_iter()
            .map(|(basic, specific)| {
                let targets: Vec<Address> = targets_by_grant
                    .get(&specific.id)
                    .map(|v| v.as_slice())
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|t| {
                        let arr: [u8; 20] = t.address.clone().try_into().ok()?;
                        Some(Address::from(arr))
                    })
                    .collect();

                let limit = limits_by_id
                    .get(&specific.limit_id)
                    .ok_or(diesel::result::Error::NotFound)?;

                Ok(Grant {
                    id: specific.id,
                    shared_grant_id: specific.basic_grant_id,
                    shared: SharedGrantSettings::try_from_model(basic)?,
                    settings: Settings {
                        target: targets,
                        limit: VolumeRateLimit {
                            max_volume: utils::try_bytes_to_u256(&limit.max_volume).map_err(
                                |e| diesel::result::Error::DeserializationError(Box::new(e)),
                            )?,
                            window: Duration::seconds(limit.window_secs as i64),
                        },
                    },
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
