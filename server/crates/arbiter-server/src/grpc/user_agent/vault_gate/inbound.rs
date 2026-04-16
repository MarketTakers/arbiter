use arbiter_proto::proto::user_agent::{
    user_agent_request::Payload as UserAgentRequestPayload,
    vault::{
        self as proto_vault,
        bootstrap::{self as proto_bootstrap},
        request::Payload as VaultRequestPayload,
        unseal::{self as proto_unseal, request::Payload as UnsealRequestPayload},
    },
};
use tonic::Status;

use crate::{
    grpc::{Convert, TryConvert},
    peers::user_agent::vault_gate::{
        self as vault_gate, HandleBootstrapEncryptedKey, HandleHandshake, HandleUnsealEncryptedKey,
    },
};

impl TryConvert for UserAgentRequestPayload {
    type Output = vault_gate::Inbound;
    type Error = Status;

    fn try_convert(self) -> Result<vault_gate::Inbound, Status> {
        match self {
            UserAgentRequestPayload::Vault(req) => req.try_convert(),
            _ => Err(Status::permission_denied(
                "Only vault operations are permitted before unsealing",
            )),
        }
    }
}

impl TryConvert for proto_vault::Request {
    type Output = vault_gate::Inbound;
    type Error = Status;

    fn try_convert(self) -> Result<vault_gate::Inbound, Status> {
        self.payload
            .ok_or_else(|| Status::invalid_argument("Missing vault request payload"))?
            .try_convert()
    }
}

impl TryConvert for VaultRequestPayload {
    type Output = vault_gate::Inbound;
    type Error = Status;

    fn try_convert(self) -> Result<vault_gate::Inbound, Status> {
        match self {
            VaultRequestPayload::QueryState(_) => Ok(vault_gate::Inbound::HandleVaultState),
            VaultRequestPayload::Unseal(req) => req.try_convert(),
            VaultRequestPayload::Bootstrap(req) => req.try_convert(),
        }
    }
}

impl TryConvert for proto_unseal::Request {
    type Output = vault_gate::Inbound;
    type Error = Status;

    fn try_convert(self) -> Result<vault_gate::Inbound, Status> {
        self.payload
            .ok_or_else(|| Status::invalid_argument("Missing unseal request payload"))?
            .try_convert()
    }
}

impl TryConvert for UnsealRequestPayload {
    type Output = vault_gate::Inbound;
    type Error = Status;

    fn try_convert(self) -> Result<vault_gate::Inbound, Status> {
        match self {
            UnsealRequestPayload::Start(start) => start.try_convert(),
            UnsealRequestPayload::EncryptedKey(key) => Ok(key.convert()),
        }
    }
}

impl TryConvert for proto_unseal::UnsealStart {
    type Output = vault_gate::Inbound;
    type Error = Status;

    fn try_convert(self) -> Result<vault_gate::Inbound, Status> {
        let bytes = <[u8; 32]>::try_from(self.client_pubkey)
            .map_err(|_| Status::invalid_argument("Invalid X25519 public key"))?;
        Ok(vault_gate::Inbound::HandleHandshake(HandleHandshake {
            client_pubkey: x25519_dalek::PublicKey::from(bytes),
        }))
    }
}

impl Convert for proto_unseal::UnsealEncryptedKey {
    type Output = vault_gate::Inbound;

    fn convert(self) -> vault_gate::Inbound {
        vault_gate::Inbound::HandleUnsealEncryptedKey(HandleUnsealEncryptedKey {
            nonce: self.nonce,
            ciphertext: self.ciphertext,
            associated_data: self.associated_data,
        })
    }
}

impl TryConvert for proto_bootstrap::Request {
    type Output = vault_gate::Inbound;
    type Error = Status;

    fn try_convert(self) -> Result<vault_gate::Inbound, Status> {
        self.encrypted_key
            .ok_or_else(|| Status::invalid_argument("Missing bootstrap encrypted key"))?
            .try_convert()
    }
}

impl TryConvert for proto_bootstrap::BootstrapEncryptedKey {
    type Output = vault_gate::Inbound;
    type Error = Status;

    fn try_convert(self) -> Result<vault_gate::Inbound, Status> {
        Ok(vault_gate::Inbound::HandleBootstrapEncryptedKey(
            HandleBootstrapEncryptedKey {
                nonce: self.nonce,
                ciphertext: self.ciphertext,
                associated_data: self.associated_data,
            },
        ))
    }
}
