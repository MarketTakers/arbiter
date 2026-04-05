use alloy::primitives::U256;
use arbiter_proto::proto::{
    evm::{
        EvmError as ProtoEvmError,
        evm_sign_transaction_response::Result as EvmSignTransactionResult,
    },
    shared::evm::{
        EvalViolation as ProtoEvalViolation, GasLimitExceededViolation, NoMatchingGrantError,
        PolicyViolationsError, SpecificMeaning as ProtoSpecificMeaning,
        TokenInfo as ProtoTokenInfo, TransactionEvalError as ProtoTransactionEvalError,
        eval_violation::Kind as ProtoEvalViolationKind,
        specific_meaning::Meaning as ProtoSpecificMeaningKind,
        transaction_eval_error::Kind as ProtoTransactionEvalErrorKind,
    },
};

use crate::{
    evm::{
        PolicyError, VetError,
        policies::{EvalViolation, SpecificMeaning},
    },
    grpc::Convert,
};

fn u256_to_proto_bytes(value: U256) -> Vec<u8> {
    value.to_be_bytes::<32>().to_vec()
}

impl Convert for SpecificMeaning {
    type Output = ProtoSpecificMeaning;

    fn convert(self) -> Self::Output {
        let kind = match self {
            SpecificMeaning::EtherTransfer(meaning) => ProtoSpecificMeaningKind::EtherTransfer(
                arbiter_proto::proto::shared::evm::EtherTransferMeaning {
                    to: meaning.to.to_vec(),
                    value: u256_to_proto_bytes(meaning.value),
                },
            ),
            SpecificMeaning::TokenTransfer(meaning) => ProtoSpecificMeaningKind::TokenTransfer(
                arbiter_proto::proto::shared::evm::TokenTransferMeaning {
                    token: Some(ProtoTokenInfo {
                        symbol: meaning.token.symbol.to_string(),
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
            EvalViolation::InvalidTarget { target } => {
                ProtoEvalViolationKind::InvalidTarget(target.to_vec())
            }
            EvalViolation::GasLimitExceeded {
                max_gas_fee_per_gas,
                max_priority_fee_per_gas,
            } => ProtoEvalViolationKind::GasLimitExceeded(GasLimitExceededViolation {
                max_gas_fee_per_gas: max_gas_fee_per_gas.map(u256_to_proto_bytes),
                max_priority_fee_per_gas: max_priority_fee_per_gas.map(u256_to_proto_bytes),
            }),
            EvalViolation::RateLimitExceeded => ProtoEvalViolationKind::RateLimitExceeded(()),
            EvalViolation::VolumetricLimitExceeded => {
                ProtoEvalViolationKind::VolumetricLimitExceeded(())
            }
            EvalViolation::InvalidTime => ProtoEvalViolationKind::InvalidTime(()),
            EvalViolation::InvalidTransactionType => {
                ProtoEvalViolationKind::InvalidTransactionType(())
            }
        };

        ProtoEvalViolation { kind: Some(kind) }
    }
}

impl Convert for VetError {
    type Output = EvmSignTransactionResult;

    fn convert(self) -> Self::Output {
        let kind = match self {
            VetError::ContractCreationNotSupported => {
                ProtoTransactionEvalErrorKind::ContractCreationNotSupported(())
            }
            VetError::UnsupportedTransactionType => {
                ProtoTransactionEvalErrorKind::UnsupportedTransactionType(())
            }
            VetError::Evaluated(meaning, policy_error) => match policy_error {
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
                PolicyError::Database(_)| PolicyError::Integrity(_) => {
                    return EvmSignTransactionResult::Error(ProtoEvmError::Internal.into());
                }
            },
        };

        EvmSignTransactionResult::EvalError(ProtoTransactionEvalError { kind: Some(kind) })
    }
}
