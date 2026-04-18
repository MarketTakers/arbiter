use crate::{
    evm::{
        PolicyError, VetError,
        policies::{EvalViolation, SpecificMeaning},
    },
    grpc::Convert,
};
use arbiter_proto::proto::{
    evm::{
        EvmError as ProtoEvmError,
        evm_sign_transaction_response::Result as EvmSignTransactionResult,
    },
    shared::evm::{
        EvalViolation as ProtoEvalViolation, GasLimitExceededViolation, NoMatchingGrantError,
        PolicyViolationsError, SpecificMeaning as ProtoSpecificMeaning,
        TokenInfo as ProtoTokenInfo, TransactionEvalError as ProtoTransactionEvalError,
        eval_violation as proto_eval_violation, eval_violation::Kind as ProtoEvalViolationKind,
        specific_meaning::Meaning as ProtoSpecificMeaningKind,
        transaction_eval_error::Kind as ProtoTransactionEvalErrorKind,
    },
};

use alloy::primitives::U256;

fn u256_to_proto_bytes(value: U256) -> Vec<u8> {
    value.to_be_bytes::<32>().to_vec()
}

impl Convert for SpecificMeaning {
    type Output = ProtoSpecificMeaning;

    fn convert(self) -> Self::Output {
        let kind = match self {
            Self::EtherTransfer(meaning) => ProtoSpecificMeaningKind::EtherTransfer(
                arbiter_proto::proto::shared::evm::EtherTransferMeaning {
                    to: meaning.to.to_vec(),
                    value: u256_to_proto_bytes(meaning.value),
                },
            ),
            Self::TokenTransfer(meaning) => ProtoSpecificMeaningKind::TokenTransfer(
                arbiter_proto::proto::shared::evm::TokenTransferMeaning {
                    token: Some(ProtoTokenInfo {
                        symbol: meaning.token.symbol.to_owned(),
                        address: meaning.token.contract.to_vec(),
                        chain_id: meaning.token.chain,
                    }),
                    to: meaning.to.to_vec(),
                    value: u256_to_proto_bytes(meaning.value),
                },
            ),
        };

        ProtoSpecificMeaning {
            meaning: Some(kind),
        }
    }
}

impl Convert for EvalViolation {
    type Output = ProtoEvalViolation;

    fn convert(self) -> Self::Output {
        let kind = match self {
            Self::InvalidTarget { target } => {
                ProtoEvalViolationKind::InvalidTarget(target.to_vec())
            }
            Self::GasLimitExceeded {
                max_gas_fee_per_gas,
                max_priority_fee_per_gas,
            } => ProtoEvalViolationKind::GasLimitExceeded(GasLimitExceededViolation {
                max_gas_fee_per_gas: max_gas_fee_per_gas.map(u256_to_proto_bytes),
                max_priority_fee_per_gas: max_priority_fee_per_gas.map(u256_to_proto_bytes),
            }),
            Self::RateLimitExceeded => ProtoEvalViolationKind::RateLimitExceeded(()),
            Self::VolumetricLimitExceeded => ProtoEvalViolationKind::VolumetricLimitExceeded(()),
            Self::InvalidTime => ProtoEvalViolationKind::InvalidTime(()),
            Self::InvalidTransactionType => ProtoEvalViolationKind::InvalidTransactionType(()),
            Self::MismatchingChainId { expected, actual } => {
                ProtoEvalViolationKind::ChainIdMismatch(proto_eval_violation::ChainIdMismatch {
                    expected,
                    actual,
                })
            }
        };

        ProtoEvalViolation { kind: Some(kind) }
    }
}

impl Convert for VetError {
    type Output = EvmSignTransactionResult;

    fn convert(self) -> Self::Output {
        let kind = match self {
            Self::ContractCreationNotSupported => {
                ProtoTransactionEvalErrorKind::ContractCreationNotSupported(())
            }
            Self::UnsupportedTransactionType => {
                ProtoTransactionEvalErrorKind::UnsupportedTransactionType(())
            }
            Self::Evaluated(meaning, policy_error) => match policy_error {
                PolicyError::NoMatchingGrant => {
                    ProtoTransactionEvalErrorKind::NoMatchingGrant(NoMatchingGrantError {
                        meaning: Some(meaning.convert()),
                    })
                }
                PolicyError::Violations(violations) => {
                    ProtoTransactionEvalErrorKind::PolicyViolations(PolicyViolationsError {
                        meaning: Some(meaning.convert()),
                        violations: violations.into_iter().map(Convert::convert).collect(),
                    })
                }
                PolicyError::Database(_) | PolicyError::Integrity(_) => {
                    return EvmSignTransactionResult::Error(ProtoEvmError::Internal.into());
                }
            },
        };

        EvmSignTransactionResult::EvalError(ProtoTransactionEvalError { kind: Some(kind) })
    }
}
