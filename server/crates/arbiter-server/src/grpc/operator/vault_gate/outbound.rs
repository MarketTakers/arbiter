use crate::{
    actors::vault::VaultState,
    grpc::{Convert, TryConvert},
    peers::operator::vault_gate::{self as vault_gate},
};
use arbiter_proto::proto::{
    operator::{
        operator_response::Payload as OperatorResponsePayload,
        vault::{
            self as proto_vault,
            bootstrap::{self as proto_bootstrap, BootstrapResult as ProtoBootstrapResult},
            response::Payload as VaultResponsePayload,
            unseal::{
                self as proto_unseal, UnsealResult as ProtoUnsealResult,
                response::Payload as UnsealResponsePayload,
            },
        },
    },
    shared::VaultState as ProtoVaultState,
};

use tonic::Status;
use tracing::warn;

const fn wrap_vault_response(payload: VaultResponsePayload) -> OperatorResponsePayload {
    OperatorResponsePayload::Vault(proto_vault::Response {
        payload: Some(payload),
    })
}

const fn wrap_unseal_response(payload: UnsealResponsePayload) -> OperatorResponsePayload {
    wrap_vault_response(VaultResponsePayload::Unseal(proto_unseal::Response {
        payload: Some(payload),
    }))
}

fn wrap_bootstrap_response(result: ProtoBootstrapResult) -> OperatorResponsePayload {
    wrap_vault_response(VaultResponsePayload::Bootstrap(proto_bootstrap::Response {
        result: result.into(),
    }))
}

impl Convert for VaultState {
    type Output = OperatorResponsePayload;

    fn convert(self) -> OperatorResponsePayload {
        let proto_state = match self {
            Self::Unbootstrapped => ProtoVaultState::Unbootstrapped,
            Self::Sealed => ProtoVaultState::Sealed,
            Self::Unsealed => ProtoVaultState::Unsealed,
        };
        wrap_vault_response(VaultResponsePayload::State(proto_state.into()))
    }
}

impl Convert for vault_gate::HandshakeResponse {
    type Output = OperatorResponsePayload;

    fn convert(self) -> OperatorResponsePayload {
        wrap_unseal_response(UnsealResponsePayload::Start(
            proto_unseal::UnsealStartResponse {
                server_pubkey: self.server_pubkey.as_bytes().to_vec(),
            },
        ))
    }
}

impl TryConvert for vault_gate::Outbound {
    type Output = OperatorResponsePayload;
    type Error = Status;

    fn try_convert(self) -> Result<OperatorResponsePayload, Status> {
        match self {
            Self::HandleVaultState(result) => result
                .map_err(|err| {
                    warn!(?err, "vault state query failed");
                    Status::internal("Failed to query vault state")
                })
                .map(VaultState::convert),
            Self::HandleHandshake(result) => result
                .map_err(|err| {
                    warn!(?err, "handshake failed");
                    Status::internal("Failed to start unseal flow")
                })
                .map(vault_gate::HandshakeResponse::convert),
            Self::HandleUnsealEncryptedKey(result) => {
                let proto_result = match result {
                    Ok(()) => ProtoUnsealResult::Success,
                    Err(vault_gate::Error::InvalidKey) => ProtoUnsealResult::InvalidKey,
                    Err(err) => {
                        warn!(?err, "unseal failed");
                        return Err(Status::internal("Failed to unseal vault"));
                    }
                };
                Ok(wrap_unseal_response(UnsealResponsePayload::Result(
                    proto_result.into(),
                )))
            }
            Self::HandleBootstrapEncryptedKey(result) => {
                let proto_result = match result {
                    Ok(()) => ProtoBootstrapResult::Success,
                    Err(vault_gate::Error::InvalidKey) => ProtoBootstrapResult::InvalidKey,
                    Err(vault_gate::Error::AlreadyBootstrapped) => {
                        ProtoBootstrapResult::AlreadyBootstrapped
                    }
                    Err(err) => {
                        warn!(?err, "bootstrap failed");
                        return Err(Status::internal("Failed to bootstrap vault"));
                    }
                };
                Ok(wrap_bootstrap_response(proto_result))
            }
            Self::HandleDeclareCommittee(result) => {
                let proto_result = match result {
                    Ok(()) => ProtoBootstrapResult::Success,
                    Err(err) => {
                        warn!(?err, "declare committee failed");
                        return Err(Status::internal("Failed to declare committee"));
                    }
                };
                Ok(wrap_bootstrap_response(proto_result))
            }
            Self::HandleContributeBootstrapPassphrase(result) => {
                let proto_result = match result {
                    Ok(true) => ProtoBootstrapResult::Success,
                    Ok(false) => ProtoBootstrapResult::AwaitingContributions,
                    Err(err) => {
                        warn!(?err, "contribute bootstrap passphrase failed");
                        return Err(Status::internal("Failed to contribute bootstrap passphrase"));
                    }
                };
                Ok(wrap_bootstrap_response(proto_result))
            }
            Self::HandleContributeRecoveryBootstrapPassphrase(result) => {
                let proto_result = match result {
                    Ok(true) => ProtoBootstrapResult::Success,
                    Ok(false) => ProtoBootstrapResult::AwaitingContributions,
                    Err(err) => {
                        warn!(?err, "contribute recovery bootstrap passphrase failed");
                        return Err(Status::internal(
                            "Failed to contribute recovery bootstrap passphrase",
                        ));
                    }
                };
                Ok(wrap_bootstrap_response(proto_result))
            }
            Self::HandleContributeUnsealPassphrase(result) => {
                let proto_result = match result {
                    Ok(true) => ProtoUnsealResult::Success,
                    Ok(false) => ProtoUnsealResult::AwaitingContributions,
                    Err(err) => {
                        warn!(?err, "contribute unseal passphrase failed");
                        return Err(Status::internal("Failed to contribute unseal passphrase"));
                    }
                };
                Ok(wrap_unseal_response(UnsealResponsePayload::Result(
                    proto_result.into(),
                )))
            }
            Self::HandleContributeRecoveryUnsealPassphrase(result) => {
                let proto_result = match result {
                    Ok(true) => ProtoUnsealResult::Success,
                    Ok(false) => ProtoUnsealResult::AwaitingContributions,
                    Err(err) => {
                        warn!(?err, "contribute recovery unseal passphrase failed");
                        return Err(Status::internal(
                            "Failed to contribute recovery unseal passphrase",
                        ));
                    }
                };
                Ok(wrap_unseal_response(UnsealResponsePayload::Result(
                    proto_result.into(),
                )))
            }
        }
    }
}
