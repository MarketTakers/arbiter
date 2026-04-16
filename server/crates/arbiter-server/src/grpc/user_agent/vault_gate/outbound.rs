use arbiter_proto::proto::{
    shared::VaultState as ProtoVaultState,
    user_agent::{
        user_agent_response::Payload as UserAgentResponsePayload,
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
};
use tonic::Status;
use tracing::warn;

use crate::{
    actors::vault::VaultState,
    grpc::{Convert, TryConvert},
    peers::user_agent::vault_gate::{self as vault_gate},
};

fn wrap_vault_response(payload: VaultResponsePayload) -> UserAgentResponsePayload {
    UserAgentResponsePayload::Vault(proto_vault::Response {
        payload: Some(payload),
    })
}

fn wrap_unseal_response(payload: UnsealResponsePayload) -> UserAgentResponsePayload {
    wrap_vault_response(VaultResponsePayload::Unseal(proto_unseal::Response {
        payload: Some(payload),
    }))
}

fn wrap_bootstrap_response(result: ProtoBootstrapResult) -> UserAgentResponsePayload {
    wrap_vault_response(VaultResponsePayload::Bootstrap(proto_bootstrap::Response {
        result: result.into(),
    }))
}

impl Convert for VaultState {
    type Output = UserAgentResponsePayload;

    fn convert(self) -> UserAgentResponsePayload {
        let proto_state = match self {
            VaultState::Unbootstrapped => ProtoVaultState::Unbootstrapped,
            VaultState::Sealed => ProtoVaultState::Sealed,
            VaultState::Unsealed => ProtoVaultState::Unsealed,
        };
        wrap_vault_response(VaultResponsePayload::State(proto_state.into()))
    }
}

impl Convert for vault_gate::HandshakeResponse {
    type Output = UserAgentResponsePayload;

    fn convert(self) -> UserAgentResponsePayload {
        wrap_unseal_response(UnsealResponsePayload::Start(
            proto_unseal::UnsealStartResponse {
                server_pubkey: self.server_pubkey.as_bytes().to_vec(),
            },
        ))
    }
}

impl TryConvert for vault_gate::Outbound {
    type Output = UserAgentResponsePayload;
    type Error = Status;

    fn try_convert(self) -> Result<UserAgentResponsePayload, Status> {
        match self {
            vault_gate::Outbound::HandleVaultState(result) => result
                .map_err(|err| {
                    warn!(?err, "vault state query failed");
                    Status::internal("Failed to query vault state")
                })
                .map(VaultState::convert),
            vault_gate::Outbound::HandleHandshake(result) => result
                .map_err(|err| {
                    warn!(?err, "handshake failed");
                    Status::internal("Failed to start unseal flow")
                })
                .map(vault_gate::HandshakeResponse::convert),
            vault_gate::Outbound::HandleUnsealEncryptedKey(result) => {
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
            vault_gate::Outbound::HandleBootstrapEncryptedKey(result) => {
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
        }
    }
}
