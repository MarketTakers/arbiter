use arbiter_proto::proto::user_agent::{
    user_agent_response::Payload as UserAgentResponsePayload,
    vault::{
        self as proto_vault,
        bootstrap::{
            self as proto_bootstrap, BootstrapEncryptedKey as ProtoBootstrapEncryptedKey,
            BootstrapResult as ProtoBootstrapResult,
        },
        request::Payload as VaultRequestPayload,
        response::Payload as VaultResponsePayload,
        unseal::{
            self as proto_unseal, UnsealEncryptedKey as ProtoUnsealEncryptedKey,
            UnsealResult as ProtoUnsealResult, UnsealStart,
            request::Payload as UnsealRequestPayload,
            response::Payload as UnsealResponsePayload,
        },
    },
};
use arbiter_proto::proto::shared::VaultState as ProtoVaultState;
use kameo::{actor::ActorRef, error::SendError};
use tonic::Status;
use tracing::warn;

use crate::{
    actors::{
        keyholder::KeyHolderState,
        user_agent::{
            UserAgentSession,
            session::connection::{
                BootstrapError, HandleBootstrapEncryptedKey, HandleQueryVaultState,
                HandleUnsealEncryptedKey, HandleUnsealRequest, UnsealError,
            },
        },
    },
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
    actor: &ActorRef<UserAgentSession>,
    req: proto_vault::Request,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let Some(payload) = req.payload else {
        return Err(Status::invalid_argument("Missing vault request payload"));
    };

    match payload {
        VaultRequestPayload::QueryState(_) => handle_query_vault_state(actor).await,
        VaultRequestPayload::Unseal(req) => dispatch_unseal_request(actor, req).await,
        VaultRequestPayload::Bootstrap(req) => handle_bootstrap_request(actor, req).await,
    }
}

async fn dispatch_unseal_request(
    actor: &ActorRef<UserAgentSession>,
    req: proto_unseal::Request,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let Some(payload) = req.payload else {
        return Err(Status::invalid_argument("Missing unseal request payload"));
    };

    match payload {
        UnsealRequestPayload::Start(req) => handle_unseal_start(actor, req).await,
        UnsealRequestPayload::EncryptedKey(req) => handle_unseal_encrypted_key(actor, req).await,
    }
}

async fn handle_unseal_start(
    actor: &ActorRef<UserAgentSession>,
    req: UnsealStart,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let client_pubkey = <[u8; 32]>::try_from(req.client_pubkey)
        .map(x25519_dalek::PublicKey::from)
        .map_err(|_| Status::invalid_argument("Invalid X25519 public key"))?;

    let response = actor
        .ask(HandleUnsealRequest { client_pubkey })
        .await
        .map_err(|err| {
            warn!(error = ?err, "Failed to handle unseal start request");
            Status::internal("Failed to start unseal flow")
        })?;

    Ok(Some(wrap_unseal_response(UnsealResponsePayload::Start(
        proto_unseal::UnsealStartResponse {
            server_pubkey: response.server_pubkey.as_bytes().to_vec(),
        },
    ))))
}

async fn handle_unseal_encrypted_key(
    actor: &ActorRef<UserAgentSession>,
    req: ProtoUnsealEncryptedKey,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let result = match actor
        .ask(HandleUnsealEncryptedKey {
            nonce: req.nonce,
            ciphertext: req.ciphertext,
            associated_data: req.associated_data,
        })
        .await
    {
        Ok(()) => ProtoUnsealResult::Success,
        Err(SendError::HandlerError(UnsealError::InvalidKey)) => ProtoUnsealResult::InvalidKey,
        Err(err) => {
            warn!(error = ?err, "Failed to handle unseal request");
            return Err(Status::internal("Failed to unseal vault"));
        }
    };
    Ok(Some(wrap_unseal_response(UnsealResponsePayload::Result(
        result.into(),
    ))))
}

async fn handle_bootstrap_request(
    actor: &ActorRef<UserAgentSession>,
    req: proto_bootstrap::Request,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let encrypted_key = req
        .encrypted_key
        .ok_or_else(|| Status::invalid_argument("Missing bootstrap encrypted key"))?;
    handle_bootstrap_encrypted_key(actor, encrypted_key).await
}

async fn handle_bootstrap_encrypted_key(
    actor: &ActorRef<UserAgentSession>,
    req: ProtoBootstrapEncryptedKey,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let result = match actor
        .ask(HandleBootstrapEncryptedKey {
            nonce: req.nonce,
            ciphertext: req.ciphertext,
            associated_data: req.associated_data,
        })
        .await
    {
        Ok(()) => ProtoBootstrapResult::Success,
        Err(SendError::HandlerError(BootstrapError::InvalidKey)) => ProtoBootstrapResult::InvalidKey,
        Err(SendError::HandlerError(BootstrapError::AlreadyBootstrapped)) => {
            ProtoBootstrapResult::AlreadyBootstrapped
        }
        Err(err) => {
            warn!(error = ?err, "Failed to handle bootstrap request");
            return Err(Status::internal("Failed to bootstrap vault"));
        }
    };
    Ok(Some(wrap_bootstrap_response(result)))
}

async fn handle_query_vault_state(
    actor: &ActorRef<UserAgentSession>,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let state = match actor.ask(HandleQueryVaultState {}).await {
        Ok(KeyHolderState::Unbootstrapped) => ProtoVaultState::Unbootstrapped,
        Ok(KeyHolderState::Sealed) => ProtoVaultState::Sealed,
        Ok(KeyHolderState::Unsealed) => ProtoVaultState::Unsealed,
        Err(err) => {
            warn!(error = ?err, "Failed to query vault state");
            ProtoVaultState::Error
        }
    };
    Ok(Some(wrap_vault_response(VaultResponsePayload::State(
        state.into(),
    ))))
}
