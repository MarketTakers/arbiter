use std::{fmt::Display, time::Duration};

use alloy::primitives::{Address, U256};
use chrono::{DateTime, Utc};
use diesel::dsl::insert_into;
use diesel::sqlite::Sqlite;
use diesel::{ExpressionMethods, JoinOnDsl, prelude::*};
use diesel_async::{AsyncConnection, RunQueryDsl};

use crate::db::models::{
    EvmEtherTransferGrant, EvmEtherTransferGrantTarget, EvmEtherTransferVolumeLimit, SqliteTimestamp,
};
use crate::evm::policies::{GrantMetadata, SpecificGrant, SpecificMeaning};
use crate::{
    db::{
        models::{
            self, NewEvmEtherTransferGrant, NewEvmEtherTransferGrantTarget,
            NewEvmEtherTransferVolumeLimit,
        },
        schema::{
            evm_ether_transfer_grant, evm_ether_transfer_grant_target,
            evm_ether_transfer_volume_limit,
        },
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VolumeLimit {
    window: Duration,
    max_volume: U256,
}

// A grant for ether transfers, which can be scoped to specific target addresses and volume limits
pub struct Grant {
    target: Vec<Address>,
    limits: Vec<VolumeLimit>,
}

impl Into<SpecificGrant> for Grant {
    fn into(self) -> SpecificGrant {
        SpecificGrant::EtherTransfer(self)
    }
}

async fn query_relevant_past_transaction(
    grant_id: i32,
    longest_window: Duration,
    db: &mut impl AsyncConnection<Backend = Sqlite>,
) -> QueryResult<Vec<(U256, DateTime<Utc>)>> {
    use crate::db::schema::evm_ether_transfer_log;
    let past_transactions: Vec<(Vec<u8>, SqliteTimestamp)> =
        evm_ether_transfer_log::table
            .filter(evm_ether_transfer_log::grant_id.eq(grant_id))
            .filter(
                evm_ether_transfer_log::created_at
                    .ge(SqliteTimestamp(chrono::Utc::now() - longest_window)),
            )
            .select((
                evm_ether_transfer_log::value,
                evm_ether_transfer_log::created_at,
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
    grant: &Grant,
    meta: &GrantMetadata,
    db: &mut impl AsyncConnection<Backend = Sqlite>,
) -> QueryResult<Vec<EvalViolation>> {
    let mut violations = Vec::new();
    // This has double meaning: checks for limit presence, and finds biggest window
    // to extract all needed historical transactions in one go later
    let longest_window = grant.limits.iter().map(|limit| limit.window).max();

    if let Some(longest_window) = longest_window {
        let _past_transaction = query_relevant_past_transaction(meta.policy_grant_id, longest_window, db).await?;

        for limit in &grant.limits {
            let window_start = chrono::Utc::now() - limit.window;
            let cumulative_volume: U256 = _past_transaction
                .iter()
                .filter(|(_, timestamp)| timestamp >= &window_start)
                .fold(U256::default(), |acc, (value, _)| acc + *value);

            if cumulative_volume > limit.max_volume {
                violations.push(EvalViolation::VolumetricLimitExceeded);
            }
        }
        // TODO: Implement actual rate limit checking logic
    }

    Ok(violations)
}

pub struct EtherTransfer;
impl Policy for EtherTransfer {
    type Grant = Grant;

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
        meaning: &Self::Meaning,
        grant: &Self::Grant,
        meta: &GrantMetadata,
        db: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> QueryResult<Vec<EvalViolation>> {
        let mut violations = Vec::new();

        // Check if the target address is within the grant's allowed targets
        if !grant.target.contains(&meaning.to) {
            violations.push(EvalViolation::InvalidTarget { target: meaning.to });
        }

        let rate_violations = check_rate_limits(grant, meta, db).await?;
        violations.extend(rate_violations);

        Ok(violations)
    }

    async fn create_grant(
        basic: &models::EvmBasicGrant,
        grant: &Self::Grant,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> diesel::result::QueryResult<DatabaseID> {
        let grant_id: i32 = insert_into(evm_ether_transfer_grant::table)
            .values(&NewEvmEtherTransferGrant {
                basic_grant_id: basic.id,
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

        for limit in &grant.limits {
            insert_into(evm_ether_transfer_volume_limit::table)
                .values(NewEvmEtherTransferVolumeLimit {
                    grant_id,
                    window_secs: limit.window.as_secs() as i32,
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
    ) -> diesel::result::QueryResult<Option<(Self::Grant, GrantMetadata)>> {
        use crate::db::schema::{
            evm_basic_grant, evm_ether_transfer_grant, evm_ether_transfer_grant_target,
        };

        let target_bytes = context.to.to_vec();

        // Find a grant where:
        // 1. The basic grant's wallet_id and client_id match the context
        // 2. Any of the grant's targets match the context's `to` address
        let grant: Option<EvmEtherTransferGrant> = evm_ether_transfer_grant::table
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
            .select(EvmEtherTransferGrant::as_select())
            .first::<EvmEtherTransferGrant>(conn)
            .await
            .optional()?;

        let Some(grant) = grant else {
            return Ok(None);
        };

        use crate::db::schema::evm_ether_transfer_volume_limit;

        // Load grant targets
        let target_bytes: Vec<EvmEtherTransferGrantTarget> = evm_ether_transfer_grant_target::table
            .select(EvmEtherTransferGrantTarget::as_select())
            .filter(evm_ether_transfer_grant_target::grant_id.eq(grant.id))
            .load(conn)
            .await?;

        // Load volume limits
        let limit_rows: Vec<EvmEtherTransferVolumeLimit> = evm_ether_transfer_volume_limit::table
            .filter(evm_ether_transfer_volume_limit::grant_id.eq(grant.id))
            .select(EvmEtherTransferVolumeLimit::as_select())
            .load(conn)
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

        // Convert database rows to VolumeLimit
        let limits: Vec<VolumeLimit> = limit_rows
            .into_iter()
            .filter_map(|limit| {
                // TODO: Handle invalid volumes more gracefully
                let max_volume = utils::bytes_to_u256(&limit.max_volume)?;
                Some(VolumeLimit {
                    window: Duration::from_secs(limit.window_secs as u64),
                    max_volume,
                })
            })
            .collect();

        let domain_grant = Grant {
            target: targets,
            limits,
        };

        Ok(Some((
            domain_grant,
            GrantMetadata {
                basic_grant_id: grant.basic_grant_id,
                policy_grant_id: grant.id,
            },
        )))
    }

    async fn record_transaction(
        context: &EvalContext,
        grant: &GrantMetadata,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> diesel::result::QueryResult<()> {
        use crate::db::schema::evm_ether_transfer_log;

        insert_into(evm_ether_transfer_log::table)
            .values(models::NewEvmEtherTransferLog {
                grant_id: grant.policy_grant_id,
                value: utils::u256_to_bytes(context.value).to_vec(),
                client_id: context.client_id,
                wallet_id: context.wallet_id,
                chain_id: context.chain as i32,
                recipient_address: context.to.to_vec(),
            })
            .execute(conn)
            .await?;

        Ok(())
    }
}
