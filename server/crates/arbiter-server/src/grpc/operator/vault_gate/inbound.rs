use crate::{
    grpc::{Convert, TryConvert},
    peers::operator::vault_gate::{
        self as vault_gate, HandleBootstrapEncryptedKey, HandleContributeBootstrapPassphrase,
        HandleContributeRecoveryBootstrapPassphrase, HandleContributeRecoveryUnsealPassphrase,
        HandleContributeUnsealPassphrase, HandleDeclareCommittee, HandleHandshake,
        HandleUnsealEncryptedKey,
    },
};
use arbiter_proto::proto::operator::{
    operator_request::Payload as OperatorRequestPayload,
    vault::{
        self as proto_vault,
        bootstrap::{self as proto_bootstrap, request::Payload as BootstrapRequestPayload},
        request::Payload as VaultRequestPayload,
        unseal::{self as proto_unseal, request::Payload as UnsealRequestPayload},
    },
};

use tonic::Status;

impl TryConvert for OperatorRequestPayload {
    type Output = vault_gate::Inbound;
    type Error = Status;

    fn try_convert(self) -> Result<vault_gate::Inbound, Status> {
        match self {
            Self::Vault(req) => req.try_convert(),
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
            Self::QueryState(()) => Ok(vault_gate::Inbound::HandleVaultState),
            Self::Unseal(req) => req.try_convert(),
            Self::Bootstrap(req) => req.try_convert(),
            Self::Rekey(_) => Err(Status::permission_denied(
                "Rekey requires an authenticated session",
            )),
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
            Self::Start(start) => start.try_convert(),
            Self::EncryptedKey(key) => Ok(key.convert()),
            Self::ContributePassphrase(cp) => Ok(
                vault_gate::Inbound::HandleContributeUnsealPassphrase(
                    HandleContributeUnsealPassphrase {
                        passphrase: cp.passphrase,
                    },
                ),
            ),
            Self::ContributeRecoveryPassphrase(crp) => Ok(
                vault_gate::Inbound::HandleContributeRecoveryUnsealPassphrase(
                    HandleContributeRecoveryUnsealPassphrase {
                        recovery_operator_id: crp.recovery_operator_id,
                        passphrase: crp.passphrase,
                    },
                ),
            ),
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
        self.payload
            .ok_or_else(|| Status::invalid_argument("Missing bootstrap payload"))?
            .try_convert()
    }
}

impl TryConvert for BootstrapRequestPayload {
    type Output = vault_gate::Inbound;
    type Error = Status;

    fn try_convert(self) -> Result<vault_gate::Inbound, Status> {
        match self {
            Self::EncryptedKey(key) => key.try_convert(),
            Self::DeclareCommittee(dc) => Ok(
                vault_gate::Inbound::HandleDeclareCommittee(HandleDeclareCommittee {
                    count: dc.count as usize,
                    recovery_count: dc.recovery_count as usize,
                }),
            ),
            Self::ContributePassphrase(cp) => Ok(
                vault_gate::Inbound::HandleContributeBootstrapPassphrase(
                    HandleContributeBootstrapPassphrase {
                        passphrase: cp.passphrase,
                    },
                ),
            ),
            Self::ContributeRecoveryPassphrase(crp) => Ok(
                vault_gate::Inbound::HandleContributeRecoveryBootstrapPassphrase(
                    HandleContributeRecoveryBootstrapPassphrase {
                        recovery_operator_id: crp.recovery_operator_id,
                        passphrase: crp.passphrase,
                    },
                ),
            ),
        }
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
