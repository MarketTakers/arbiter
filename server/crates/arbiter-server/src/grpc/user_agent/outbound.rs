use arbiter_proto::proto::{
    evm::{
        EtherTransferSettings as ProtoEtherTransferSettings, SharedSettings as ProtoSharedSettings,
        SpecificGrant as ProtoSpecificGrant, TokenTransferSettings as ProtoTokenTransferSettings,
        TransactionRateLimit as ProtoTransactionRateLimit, VolumeRateLimit as ProtoVolumeRateLimit,
        specific_grant::Grant as ProtoSpecificGrantType,
    },
    user_agent::sdk_client::{WalletAccess, WalletAccessEntry as ProtoSdkClientWalletAccess},
};
use chrono::{DateTime, Utc};
use prost_types::Timestamp as ProtoTimestamp;

use crate::{
    db::models::EvmWalletAccess,
    evm::policies::{SharedGrantSettings, SpecificGrant, TransactionRateLimit, VolumeRateLimit},
    grpc::Convert,
};

impl Convert for DateTime<Utc> {
    type Output = ProtoTimestamp;

    fn convert(self) -> ProtoTimestamp {
        ProtoTimestamp {
            seconds: self.timestamp(),
            nanos: self.timestamp_subsec_nanos().try_into().unwrap_or(i32::MAX),
        }
    }
}

impl Convert for TransactionRateLimit {
    type Output = ProtoTransactionRateLimit;

    fn convert(self) -> ProtoTransactionRateLimit {
        ProtoTransactionRateLimit {
            count: self.count,
            window_secs: self.window.num_seconds(),
        }
    }
}

impl Convert for VolumeRateLimit {
    type Output = ProtoVolumeRateLimit;

    fn convert(self) -> ProtoVolumeRateLimit {
        ProtoVolumeRateLimit {
            max_volume: self.max_volume.to_be_bytes::<32>().to_vec(),
            window_secs: self.window.num_seconds(),
        }
    }
}

impl Convert for SharedGrantSettings {
    type Output = ProtoSharedSettings;

    fn convert(self) -> ProtoSharedSettings {
        ProtoSharedSettings {
            wallet_access_id: self.wallet_access_id,
            chain_id: self.chain,
            valid_from: self.valid_from.map(DateTime::convert),
            valid_until: self.valid_until.map(DateTime::convert),
            max_gas_fee_per_gas: self
                .max_gas_fee_per_gas
                .map(|value| value.to_be_bytes::<32>().to_vec()),
            max_priority_fee_per_gas: self
                .max_priority_fee_per_gas
                .map(|value| value.to_be_bytes::<32>().to_vec()),
            rate_limit: self.rate_limit.map(TransactionRateLimit::convert),
        }
    }
}

impl Convert for SpecificGrant {
    type Output = ProtoSpecificGrant;

    fn convert(self) -> ProtoSpecificGrant {
        let grant = match self {
            Self::EtherTransfer(s) => {
                ProtoSpecificGrantType::EtherTransfer(ProtoEtherTransferSettings {
                    targets: s.target.into_iter().map(|a| a.to_vec()).collect(),
                    limit: Some(s.limit.convert()),
                })
            }
            Self::TokenTransfer(s) => {
                ProtoSpecificGrantType::TokenTransfer(ProtoTokenTransferSettings {
                    token_contract: s.token_contract.to_vec(),
                    target: s.target.map(|a| a.to_vec()),
                    volume_limits: s
                        .volume_limits
                        .into_iter()
                        .map(VolumeRateLimit::convert)
                        .collect(),
                })
            }
        };
        ProtoSpecificGrant { grant: Some(grant) }
    }
}

impl Convert for EvmWalletAccess {
    type Output = ProtoSdkClientWalletAccess;

    fn convert(self) -> Self::Output {
        Self::Output {
            id: self.id,
            access: Some(WalletAccess {
                wallet_id: self.wallet_id,
                sdk_client_id: self.client_id,
            }),
        }
    }
}
