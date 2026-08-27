//! Creating a standing EVM grant.
use super::{Proposal, ProposalKindTag, as_i64, as_u64, fixed};
use crate::db::{
    DatabaseConnection,
    models::ProposalId,
    schema::{
        proposal_persistent_grant, proposal_persistent_grant_ether,
        proposal_persistent_grant_ether_target, proposal_persistent_grant_token,
        proposal_persistent_grant_token_limit,
    },
};
use diesel::{
    ExpressionMethods as _, Insertable, OptionalExtension as _, QueryDsl as _, QueryResult,
    Queryable, Selectable, SelectableHelper as _, sqlite::Sqlite,
};
use diesel_async::RunQueryDsl as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub wallet_access_id: i32,
    pub chain_id: u64,
    pub valid_from_secs: Option<i64>,
    pub valid_until_secs: Option<i64>,
    pub max_gas_fee_per_gas: Option<[u8; 32]>,
    pub max_priority_fee_per_gas: Option<[u8; 32]>,
    pub rate_limit: Option<RateLimit>,
    pub specific: Specific,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimit {
    pub count: u32,
    pub window_secs: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeLimit {
    pub max_volume: [u8; 32],
    pub window_secs: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Specific {
    EtherTransfer {
        targets: Vec<[u8; 20]>,
        limit: VolumeLimit,
    },
    TokenTransfer {
        token_contract: [u8; 20],
        receiver: Option<[u8; 20]>,
        volume_limits: Vec<VolumeLimit>,
    },
}

/// Shared settings, mirroring `evm_basic_grant`.
#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = proposal_persistent_grant, check_for_backend(Sqlite))]
struct BaseRow {
    proposal_id: ProposalId,
    wallet_access_id: i32,
    chain_id: i64,
    valid_from: Option<i64>,
    valid_until: Option<i64>,
    max_gas_fee_per_gas: Option<Vec<u8>>,
    max_priority_fee_per_gas: Option<Vec<u8>>,
    rate_limit_count: Option<i32>,
    rate_limit_window_secs: Option<i64>,
}

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = proposal_persistent_grant_ether, check_for_backend(Sqlite))]
struct EtherRow {
    proposal_id: ProposalId,
    window_secs: i64,
    max_volume: Vec<u8>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = proposal_persistent_grant_ether_target, check_for_backend(Sqlite))]
struct NewEtherTarget {
    proposal_id: ProposalId,
    address: Vec<u8>,
}

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = proposal_persistent_grant_token, check_for_backend(Sqlite))]
struct TokenRow {
    proposal_id: ProposalId,
    token_contract: Vec<u8>,
    receiver: Option<Vec<u8>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = proposal_persistent_grant_token_limit, check_for_backend(Sqlite))]
struct NewTokenLimit {
    proposal_id: ProposalId,
    window_secs: i64,
    max_volume: Vec<u8>,
}

impl BaseRow {
    fn new(proposal_id: ProposalId, settings: &Settings) -> QueryResult<Self> {
        Ok(Self {
            proposal_id,
            wallet_access_id: settings.wallet_access_id,
            chain_id: as_i64(settings.chain_id)?,
            valid_from: settings.valid_from_secs,
            valid_until: settings.valid_until_secs,
            max_gas_fee_per_gas: settings.max_gas_fee_per_gas.map(|v| v.to_vec()),
            max_priority_fee_per_gas: settings.max_priority_fee_per_gas.map(|v| v.to_vec()),
            // SQLite stores integers signed; a rate-limit count is a `u32`, so it
            // round-trips through the bit pattern rather than a fallible range check.
            rate_limit_count: settings.rate_limit.map(|r| r.count.cast_signed()),
            rate_limit_window_secs: settings.rate_limit.map(|r| r.window_secs),
        })
    }

    fn into_settings(self, specific: Specific) -> QueryResult<Settings> {
        Ok(Settings {
            wallet_access_id: self.wallet_access_id,
            chain_id: as_u64(self.chain_id)?,
            valid_from_secs: self.valid_from,
            valid_until_secs: self.valid_until,
            max_gas_fee_per_gas: fixed!(opt self.max_gas_fee_per_gas)?,
            max_priority_fee_per_gas: fixed!(opt self.max_priority_fee_per_gas)?,
            rate_limit: self.rate_limit_count.zip(self.rate_limit_window_secs).map(
                |(count, window_secs)| RateLimit {
                    count: count.cast_unsigned(),
                    window_secs,
                },
            ),
            specific,
        })
    }
}

pub struct PersistentGrant;

impl Proposal for PersistentGrant {
    const KIND: ProposalKindTag = ProposalKindTag::ApprovePersistentGrant;

    type Settings = Settings;

    async fn insert(
        proposal_id: ProposalId,
        settings: &Self::Settings,
        conn: &mut DatabaseConnection,
    ) -> QueryResult<()> {
        diesel::insert_into(proposal_persistent_grant::table)
            .values(&BaseRow::new(proposal_id, settings)?)
            .execute(conn)
            .await?;

        match &settings.specific {
            Specific::EtherTransfer { targets, limit } => {
                diesel::insert_into(proposal_persistent_grant_ether::table)
                    .values(&EtherRow {
                        proposal_id,
                        window_secs: limit.window_secs,
                        max_volume: limit.max_volume.to_vec(),
                    })
                    .execute(conn)
                    .await?;

                // Row at a time: SQLite has no multi-row VALUES clause in diesel-async.
                for address in targets {
                    diesel::insert_into(proposal_persistent_grant_ether_target::table)
                        .values(&NewEtherTarget {
                            proposal_id,
                            address: address.to_vec(),
                        })
                        .execute(conn)
                        .await?;
                }
            }
            Specific::TokenTransfer {
                token_contract,
                receiver,
                volume_limits,
            } => {
                diesel::insert_into(proposal_persistent_grant_token::table)
                    .values(&TokenRow {
                        proposal_id,
                        token_contract: token_contract.to_vec(),
                        receiver: receiver.map(|r| r.to_vec()),
                    })
                    .execute(conn)
                    .await?;

                for limit in volume_limits {
                    diesel::insert_into(proposal_persistent_grant_token_limit::table)
                        .values(&NewTokenLimit {
                            proposal_id,
                            window_secs: limit.window_secs,
                            max_volume: limit.max_volume.to_vec(),
                        })
                        .execute(conn)
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn load(
        proposal_id: ProposalId,
        conn: &mut DatabaseConnection,
    ) -> QueryResult<Self::Settings> {
        let base: BaseRow = proposal_persistent_grant::table
            .find(proposal_id)
            .select(BaseRow::as_select())
            .first(conn)
            .await?;

        let ether: Option<EtherRow> = proposal_persistent_grant_ether::table
            .find(proposal_id)
            .select(EtherRow::as_select())
            .first(conn)
            .await
            .optional()?;

        let specific = if let Some(ether) = ether {
            let addresses: Vec<Vec<u8>> = proposal_persistent_grant_ether_target::table
                .filter(proposal_persistent_grant_ether_target::proposal_id.eq(proposal_id))
                .select(proposal_persistent_grant_ether_target::address)
                .load(conn)
                .await?;
            let targets = addresses
                .iter()
                .map(|address| fixed!(address))
                .collect::<QueryResult<Vec<_>>>()?;
            Specific::EtherTransfer {
                targets,
                limit: VolumeLimit {
                    max_volume: fixed!(ether.max_volume)?,
                    window_secs: ether.window_secs,
                },
            }
        } else {
            let token: TokenRow = proposal_persistent_grant_token::table
                .find(proposal_id)
                .select(TokenRow::as_select())
                .first(conn)
                .await?;
            let rows: Vec<(i64, Vec<u8>)> = proposal_persistent_grant_token_limit::table
                .filter(proposal_persistent_grant_token_limit::proposal_id.eq(proposal_id))
                .select((
                    proposal_persistent_grant_token_limit::window_secs,
                    proposal_persistent_grant_token_limit::max_volume,
                ))
                .load(conn)
                .await?;
            let volume_limits = rows
                .into_iter()
                .map(|(window_secs, max_volume)| {
                    Ok(VolumeLimit {
                        max_volume: fixed!(max_volume)?,
                        window_secs,
                    })
                })
                .collect::<QueryResult<Vec<_>>>()?;
            Specific::TokenTransfer {
                token_contract: fixed!(token.token_contract)?,
                receiver: fixed!(opt token.receiver)?,
                volume_limits,
            }
        };

        base.into_settings(specific)
    }
}
