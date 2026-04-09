use alloy::primitives::{Address, U256};
use arbiter_proto::proto::evm::{
    EtherTransferSettings as ProtoEtherTransferSettings, SharedSettings as ProtoSharedSettings,
    SpecificGrant as ProtoSpecificGrant, TokenTransferSettings as ProtoTokenTransferSettings,
    TransactionRateLimit as ProtoTransactionRateLimit, VolumeRateLimit as ProtoVolumeRateLimit,
    specific_grant::Grant as ProtoSpecificGrantType,
};
use arbiter_proto::proto::user_agent::sdk_client::{
    WalletAccess, WalletAccessEntry as SdkClientWalletAccess,
};
use chrono::{DateTime, TimeZone, Utc};
use prost_types::Timestamp as ProtoTimestamp;
use tonic::Status;

use crate::db::models::{CoreEvmWalletAccess, NewEvmWalletAccess};
use crate::grpc::Convert;
use crate::{
    evm::policies::{
        SharedGrantSettings, SpecificGrant, TransactionRateLimit, VolumeRateLimit, ether_transfer,
        token_transfers,
    },
    grpc::TryConvert,
};

fn address_from_bytes(bytes: &[u8]) -> Result<Address, Status> {
    if bytes.len() != 20 {
        return Err(Status::invalid_argument("Invalid EVM address"));
    }
    Ok(Address::from_slice(bytes))
}

fn u256_from_proto_bytes(bytes: &[u8]) -> Result<U256, Status> {
    if bytes.len() > 32 {
        return Err(Status::invalid_argument("Invalid U256 byte length"));
    }
    Ok(U256::from_be_slice(bytes))
}

impl TryConvert for ProtoTimestamp {
    type Output = DateTime<Utc>;
    type Error = Status;

    fn try_convert(self) -> Result<DateTime<Utc>, Status> {
        Utc.timestamp_opt(self.seconds, self.nanos.try_into().unwrap_or_default())
            .single()
            .ok_or_else(|| Status::invalid_argument("Invalid timestamp"))
    }
}

impl TryConvert for ProtoTransactionRateLimit {
    type Output = TransactionRateLimit;
    type Error = Status;

    fn try_convert(self) -> Result<TransactionRateLimit, Status> {
        Ok(TransactionRateLimit {
            count: self.count,
            window: chrono::Duration::seconds(self.window_secs),
        })
    }
}

impl TryConvert for ProtoVolumeRateLimit {
    type Output = VolumeRateLimit;
    type Error = Status;

    fn try_convert(self) -> Result<VolumeRateLimit, Status> {
        Ok(VolumeRateLimit {
            max_volume: u256_from_proto_bytes(&self.max_volume)?,
            window: chrono::Duration::seconds(self.window_secs),
        })
    }
}

impl TryConvert for ProtoSharedSettings {
    type Output = SharedGrantSettings;
    type Error = Status;

    fn try_convert(self) -> Result<SharedGrantSettings, Status> {
        Ok(SharedGrantSettings {
            wallet_access_id: self.wallet_access_id,
            chain: self.chain_id,
            valid_from: self
                .valid_from
                .map(ProtoTimestamp::try_convert)
                .transpose()?,
            valid_until: self
                .valid_until
                .map(ProtoTimestamp::try_convert)
                .transpose()?,
            max_gas_fee_per_gas: self
                .max_gas_fee_per_gas
                .as_deref()
                .map(u256_from_proto_bytes)
                .transpose()?,
            max_priority_fee_per_gas: self
                .max_priority_fee_per_gas
                .as_deref()
                .map(u256_from_proto_bytes)
                .transpose()?,
            rate_limit: self
                .rate_limit
                .map(ProtoTransactionRateLimit::try_convert)
                .transpose()?,
        })
    }
}

impl TryConvert for ProtoSpecificGrant {
    type Output = SpecificGrant;
    type Error = Status;

    fn try_convert(self) -> Result<SpecificGrant, Status> {
        match self.grant {
            Some(ProtoSpecificGrantType::EtherTransfer(ProtoEtherTransferSettings {
                targets,
                limit,
            })) => Ok(SpecificGrant::EtherTransfer(ether_transfer::Settings {
                target: targets
                    .iter()
                    .map(Vec::as_slice)
                    .map(address_from_bytes)
                    .collect::<Result<_, _>>()?,
                limit: limit
                    .ok_or_else(|| {
                        Status::invalid_argument("Missing ether transfer volume rate limit")
                    })?
                    .try_convert()?,
            })),
            Some(ProtoSpecificGrantType::TokenTransfer(ProtoTokenTransferSettings {
                token_contract,
                target,
                volume_limits,
            })) => Ok(SpecificGrant::TokenTransfer(token_transfers::Settings {
                token_contract: address_from_bytes(&token_contract)?,
                target: target
                    .map(|target| address_from_bytes(&target))
                    .transpose()?,
                volume_limits: volume_limits
                    .into_iter()
                    .map(ProtoVolumeRateLimit::try_convert)
                    .collect::<Result<_, _>>()?,
            })),
            None => Err(Status::invalid_argument("Missing specific grant kind")),
        }
    }
}

impl Convert for WalletAccess {
    type Output = NewEvmWalletAccess;

    fn convert(self) -> Self::Output {
        NewEvmWalletAccess {
            wallet_id: self.wallet_id,
            client_id: self.sdk_client_id,
        }
    }
}

impl TryConvert for SdkClientWalletAccess {
    type Output = CoreEvmWalletAccess;
    type Error = Status;

    fn try_convert(self) -> Result<CoreEvmWalletAccess, Status> {
        let Some(access) = self.access else {
            return Err(Status::invalid_argument("Missing wallet access entry"));
        };
        Ok(CoreEvmWalletAccess {
            wallet_id: access.wallet_id,
            client_id: access.sdk_client_id,
            id: self.id,
        })
    }
}
