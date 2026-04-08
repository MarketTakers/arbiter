use arbiter_proto::proto::user_agent::{
    user_agent_response::Payload as UserAgentResponsePayload,
    vault::{
        self as proto_vault,
        bootstrap::{
            self as proto_bootstrap, BootstrapResult as ProtoBootstrapResult,
        },
        request::Payload as VaultRequestPayload,
        response::Payload as VaultResponsePayload,
        unseal::{
            self as proto_unseal, UnsealResult as ProtoUnsealResult, UnsealStart,
            request::Payload as UnsealRequestPayload, response::Payload as UnsealResponsePayload,
        },
    },
};
use kameo::{actor::ActorRef, error::SendError};
use tonic::Status;
use tracing::warn;

use crate::peers::user_agent::vault_gate::{
    self as vault_gate, HandleBootstrapEncryptedKey, HandleHandshake, HandleUnsealEncryptedKey,
    VaultGate,
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

pub(super) async fn dispatch(
    gate: &ActorRef<VaultGate>,
    req: proto_vault::Request,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let Some(payload) = req.payload else {
        return Err(Status::invalid_argument("Missing vault request payload"));
    };

    match payload {
        VaultRequestPayload::QueryState(_) => {
            use arbiter_proto::proto::shared::VaultState as ProtoVaultState;
            Ok(Some(wrap_vault_response(VaultResponsePayload::State(
                ProtoVaultState::Sealed.into(),
            ))))
        }
        VaultRequestPayload::Unseal(req) => dispatch_unseal(gate, req).await,
        VaultRequestPayload::Bootstrap(req) => dispatch_bootstrap(gate, req).await,
    }
}

async fn dispatch_unseal(
    gate: &ActorRef<VaultGate>,
    req: proto_unseal::Request,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let Some(payload) = req.payload else {
        return Err(Status::invalid_argument("Missing unseal request payload"));
    };

    match payload {
        UnsealRequestPayload::Start(req) => handle_unseal_start(gate, req).await,
        UnsealRequestPayload::EncryptedKey(req) => handle_unseal_encrypted_key(gate, req).await,
    }
}

async fn handle_unseal_start(
    gate: &ActorRef<VaultGate>,
    req: UnsealStart,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let client_pubkey = <[u8; 32]>::try_from(req.client_pubkey)
        .map(x25519_dalek::PublicKey::from)
        .map_err(|_| Status::invalid_argument("Invalid X25519 public key"))?;

    let response = gate
        .ask(HandleHandshake { client_pubkey })
        .await
        .map_err(|err| {
            warn!(error = ?err, "Failed to handle unseal start");
            Status::internal("Failed to start unseal flow")
        })?;

    Ok(Some(wrap_unseal_response(UnsealResponsePayload::Start(
        proto_unseal::UnsealStartResponse {
            server_pubkey: response.server_pubkey.as_bytes().to_vec(),
        },
    ))))
}

async fn handle_unseal_encrypted_key(
    gate: &ActorRef<VaultGate>,
    req: arbiter_proto::proto::user_agent::vault::unseal::UnsealEncryptedKey,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let result = match gate
        .ask(HandleUnsealEncryptedKey {
            nonce: req.nonce,
            ciphertext: req.ciphertext,
            associated_data: req.associated_data,
        })
        .await
    {
        Ok(()) => ProtoUnsealResult::Success,
        Err(SendError::HandlerError(vault_gate::Error::InvalidKey)) => ProtoUnsealResult::InvalidKey,
        Err(err) => {
            warn!(error = ?err, "Failed to handle unseal request");
            return Err(Status::internal("Failed to unseal vault"));
        }
    };
    Ok(Some(wrap_unseal_response(UnsealResponsePayload::Result(
        result.into(),
    ))))
}

async fn dispatch_bootstrap(
    gate: &ActorRef<VaultGate>,
    req: proto_bootstrap::Request,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let encrypted_key = req
        .encrypted_key
        .ok_or_else(|| Status::invalid_argument("Missing bootstrap encrypted key"))?;

    let result = match gate
        .ask(HandleBootstrapEncryptedKey {
            nonce: encrypted_key.nonce,
            ciphertext: encrypted_key.ciphertext,
            associated_data: encrypted_key.associated_data,
        })
        .await
    {
        Ok(()) => ProtoBootstrapResult::Success,
        Err(SendError::HandlerError(vault_gate::Error::InvalidKey)) => ProtoBootstrapResult::InvalidKey,
        Err(SendError::HandlerError(vault_gate::Error::AlreadyBootstrapped)) => {
            ProtoBootstrapResult::AlreadyBootstrapped
        }
        Err(err) => {
            warn!(error = ?err, "Failed to handle bootstrap request");
            return Err(Status::internal("Failed to bootstrap vault"));
        }
    };
    Ok(Some(wrap_bootstrap_response(result)))
}
