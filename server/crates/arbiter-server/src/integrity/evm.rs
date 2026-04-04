use alloy::primitives::Address;
use arbiter_proto::proto::integrity::{
    IntegrityEtherTransferSettings, IntegrityEvmGrantPayloadV1, IntegritySharedGrantSettings,
    IntegritySpecificGrant, IntegrityTokenTransferSettings, IntegrityTransactionRateLimit,
    IntegrityVolumeRateLimit, integrity_specific_grant,
};
use chrono::{DateTime, Utc};
use diesel::sqlite::Sqlite;
use diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl, SelectableHelper as _};
use diesel_async::{AsyncConnection, RunQueryDsl};
use prost::Message;
use prost_types::Timestamp;

use crate::{
    db::{models, schema},
    evm::policies::{Grant, SharedGrantSettings, SpecificGrant, VolumeRateLimit},
    integrity::IntegrityEntity,
};

pub const EVM_GRANT_ENTITY_KIND: &str = "evm_grant";

#[derive(Debug, Clone)]
pub struct SignedEvmGrant {
    pub basic_grant_id: i32,
    pub shared: SharedGrantSettings,
    pub specific: SpecificGrant,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl SignedEvmGrant {
    pub fn from_active_grant(grant: &Grant<SpecificGrant>) -> Self {
        Self {
            basic_grant_id: grant.shared_grant_id,
            shared: grant.shared.clone(),
            specific: grant.settings.clone(),
            revoked_at: None,
        }
    }
}

fn timestamp(value: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: value.timestamp(),
        nanos: 0,
    }
}

fn encode_shared(shared: &SharedGrantSettings) -> IntegritySharedGrantSettings {
    IntegritySharedGrantSettings {
        wallet_access_id: shared.wallet_access_id,
        chain_id: shared.chain,
        valid_from: shared.valid_from.map(timestamp),
        valid_until: shared.valid_until.map(timestamp),
        max_gas_fee_per_gas: shared
            .max_gas_fee_per_gas
            .map(|v| v.to_le_bytes::<32>().to_vec()),
        max_priority_fee_per_gas: shared
            .max_priority_fee_per_gas
            .map(|v| v.to_le_bytes::<32>().to_vec()),
        rate_limit: shared
            .rate_limit
            .as_ref()
            .map(|rl| IntegrityTransactionRateLimit {
                count: rl.count,
                window_secs: rl.window.num_seconds(),
            }),
    }
}

fn encode_volume_limit(limit: &VolumeRateLimit) -> IntegrityVolumeRateLimit {
    IntegrityVolumeRateLimit {
        max_volume: limit.max_volume.to_le_bytes::<32>().to_vec(),
        window_secs: limit.window.num_seconds(),
    }
}

fn try_bytes_to_u256(bytes: &[u8]) -> diesel::result::QueryResult<alloy::primitives::U256> {
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| {
        diesel::result::Error::DeserializationError(
            format!("Expected 32-byte U256 payload, got {}", bytes.len()).into(),
        )
    })?;
    Ok(alloy::primitives::U256::from_le_bytes(bytes))
}

fn encode_specific(specific: &SpecificGrant) -> IntegritySpecificGrant {
    let grant = match specific {
        SpecificGrant::EtherTransfer(settings) => {
            let mut targets: Vec<Vec<u8>> =
                settings.target.iter().map(|addr| addr.to_vec()).collect();
            targets.sort_unstable();

            integrity_specific_grant::Grant::EtherTransfer(IntegrityEtherTransferSettings {
                targets,
                limit: Some(encode_volume_limit(&settings.limit)),
            })
        }
        SpecificGrant::TokenTransfer(settings) => {
            let mut volume_limits: Vec<IntegrityVolumeRateLimit> = settings
                .volume_limits
                .iter()
                .map(encode_volume_limit)
                .collect();
            volume_limits.sort_by(|left, right| {
                left.window_secs
                    .cmp(&right.window_secs)
                    .then_with(|| left.max_volume.cmp(&right.max_volume))
            });

            integrity_specific_grant::Grant::TokenTransfer(IntegrityTokenTransferSettings {
                token_contract: settings.token_contract.to_vec(),
                target: settings.target.map(|a| a.to_vec()),
                volume_limits,
            })
        }
    };

    IntegritySpecificGrant { grant: Some(grant) }
}

impl IntegrityEntity for SignedEvmGrant {
    fn entity_kind(&self) -> &'static str {
        EVM_GRANT_ENTITY_KIND
    }

    fn entity_id_bytes(&self) -> Vec<u8> {
        self.basic_grant_id.to_be_bytes().to_vec()
    }

    fn payload_version(&self) -> i32 {
        1
    }

    fn canonical_payload_bytes(&self) -> Vec<u8> {
        IntegrityEvmGrantPayloadV1 {
            basic_grant_id: self.basic_grant_id,
            shared: Some(encode_shared(&self.shared)),
            specific: Some(encode_specific(&self.specific)),
            revoked_at: self.revoked_at.map(timestamp),
        }
        .encode_to_vec()
    }
}

pub async fn load_signed_grant_by_basic_id(
    conn: &mut impl AsyncConnection<Backend = Sqlite>,
    basic_grant_id: i32,
) -> diesel::result::QueryResult<SignedEvmGrant> {
    let basic: models::EvmBasicGrant = schema::evm_basic_grant::table
        .filter(schema::evm_basic_grant::id.eq(basic_grant_id))
        .select(models::EvmBasicGrant::as_select())
        .first(conn)
        .await?;

    let specific_token: Option<models::EvmTokenTransferGrant> =
        schema::evm_token_transfer_grant::table
            .filter(schema::evm_token_transfer_grant::basic_grant_id.eq(basic_grant_id))
            .select(models::EvmTokenTransferGrant::as_select())
            .first(conn)
            .await
            .optional()?;

    let revoked_at = basic.revoked_at.clone().map(Into::into);
    let shared = SharedGrantSettings::try_from_model(basic)?;

    if let Some(token) = specific_token {
        let limits: Vec<models::EvmTokenTransferVolumeLimit> =
            schema::evm_token_transfer_volume_limit::table
                .filter(schema::evm_token_transfer_volume_limit::grant_id.eq(token.id))
                .select(models::EvmTokenTransferVolumeLimit::as_select())
                .load(conn)
                .await?;

        let token_contract: [u8; 20] = token.token_contract.try_into().map_err(|_| {
            diesel::result::Error::DeserializationError(
                "Invalid token contract address length".into(),
            )
        })?;

        let target = match token.receiver {
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

        let volume_limits = limits
            .into_iter()
            .map(|row| {
                Ok(VolumeRateLimit {
                    max_volume: try_bytes_to_u256(&row.max_volume)?,
                    window: chrono::Duration::seconds(row.window_secs as i64),
                })
            })
            .collect::<diesel::result::QueryResult<Vec<_>>>()?;

        return Ok(SignedEvmGrant {
            basic_grant_id,
            shared,
            specific: SpecificGrant::TokenTransfer(
                crate::evm::policies::token_transfers::Settings {
                    token_contract: Address::from(token_contract),
                    target,
                    volume_limits,
                },
            ),
            revoked_at,
        });
    }

    let ether: models::EvmEtherTransferGrant = schema::evm_ether_transfer_grant::table
        .filter(schema::evm_ether_transfer_grant::basic_grant_id.eq(basic_grant_id))
        .select(models::EvmEtherTransferGrant::as_select())
        .first(conn)
        .await?;

    let targets_rows: Vec<models::EvmEtherTransferGrantTarget> =
        schema::evm_ether_transfer_grant_target::table
            .filter(schema::evm_ether_transfer_grant_target::grant_id.eq(ether.id))
            .select(models::EvmEtherTransferGrantTarget::as_select())
            .load(conn)
            .await?;

    let limit: models::EvmEtherTransferLimit = schema::evm_ether_transfer_limit::table
        .filter(schema::evm_ether_transfer_limit::id.eq(ether.limit_id))
        .select(models::EvmEtherTransferLimit::as_select())
        .first(conn)
        .await?;

    let targets = targets_rows
        .into_iter()
        .map(|row| {
            let arr: [u8; 20] = row.address.try_into().map_err(|_| {
                diesel::result::Error::DeserializationError(
                    "Invalid ether target address length".into(),
                )
            })?;
            Ok(Address::from(arr))
        })
        .collect::<diesel::result::QueryResult<Vec<_>>>()?;

    Ok(SignedEvmGrant {
        basic_grant_id,
        shared,
        specific: SpecificGrant::EtherTransfer(crate::evm::policies::ether_transfer::Settings {
            target: targets,
            limit: VolumeRateLimit {
                max_volume: try_bytes_to_u256(&limit.max_volume)?,
                window: chrono::Duration::seconds(limit.window_secs as i64),
            },
        }),
        revoked_at,
    })
}
