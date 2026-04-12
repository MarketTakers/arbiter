use arbiter_proto::{
    proto::user_agent::{
        user_agent_request::Payload as UserAgentRequestPayload,
        user_agent_response::Payload as UserAgentResponsePayload,
        vault::{
            self as proto_vault,
            bootstrap::{self as proto_bootstrap, BootstrapResult as ProtoBootstrapResult},
            request::Payload as VaultRequestPayload,
            response::Payload as VaultResponsePayload,
            unseal::{
                self as proto_unseal, UnsealResult as ProtoUnsealResult,
                request::Payload as UnsealRequestPayload,
                response::Payload as UnsealResponsePayload,
            },
        },
    },
    transport::{Bi, Error as TransportError, Receiver, Sender},
};
use async_trait::async_trait;
use tonic::Status;
use tracing::warn;

use super::auth::AuthTransportAdapter;
use crate::peers::user_agent::vault_gate::{
    self as vault_gate, HandleBootstrapEncryptedKey, HandleHandshake, HandleUnsealEncryptedKey,
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

impl AuthTransportAdapter<'_> {
    async fn send_query_state(&mut self) -> Result<(), TransportError> {
        use arbiter_proto::proto::shared::VaultState as ProtoVaultState;
        self.send_response_payload(wrap_vault_response(VaultResponsePayload::State(
            ProtoVaultState::Sealed.into(),
        )))
        .await
    }
}

#[async_trait]
impl Receiver<vault_gate::Inbound> for AuthTransportAdapter<'_> {
    async fn recv(&mut self) -> Option<vault_gate::Inbound> {
        loop {
            let request = match self.bi_mut().recv().await? {
                Ok(request) => request,
                Err(error) => {
                    warn!(?error, "Failed to receive user agent request during vault gate");
                    return None;
                }
            };

            if let Err(err) = self.tracker_mut().request(request.id) {
                let _ = self.bi_mut().send(Err(err)).await;
                return None;
            }

            let Some(payload) = request.payload else {
                let _ = self
                    .bi_mut()
                    .send(Err(Status::invalid_argument("Missing request payload")))
                    .await;
                return None;
            };

            let vault_req = match payload {
                UserAgentRequestPayload::Vault(req) => req,
                _ => {
                    let _ = self
                        .bi_mut()
                        .send(Err(Status::permission_denied(
                            "Only vault operations are permitted before unsealing",
                        )))
                        .await;
                    return None;
                }
            };

            let Some(vault_payload) = vault_req.payload else {
                let _ = self
                    .bi_mut()
                    .send(Err(Status::invalid_argument("Missing vault request payload")))
                    .await;
                return None;
            };

            match vault_payload {
                VaultRequestPayload::QueryState(_) => {
                    if self.send_query_state().await.is_err() {
                        return None;
                    }
                    continue;
                }
                VaultRequestPayload::Unseal(req) => {
                    let Some(unseal_payload) = req.payload else {
                        let _ = self
                            .bi_mut()
                            .send(Err(Status::invalid_argument("Missing unseal request payload")))
                            .await;
                        return None;
                    };
                    match unseal_payload {
                        UnsealRequestPayload::Start(start) => {
                            let Ok(bytes) = <[u8; 32]>::try_from(start.client_pubkey) else {
                                let _ = self
                                    .bi_mut()
                                    .send(Err(Status::invalid_argument(
                                        "Invalid X25519 public key",
                                    )))
                                    .await;
                                return None;
                            };
                            return Some(vault_gate::Inbound::HandleHandshake(HandleHandshake {
                                client_pubkey: x25519_dalek::PublicKey::from(bytes),
                            }));
                        }
                        UnsealRequestPayload::EncryptedKey(key) => {
                            return Some(vault_gate::Inbound::HandleUnsealEncryptedKey(
                                HandleUnsealEncryptedKey {
                                    nonce: key.nonce,
                                    ciphertext: key.ciphertext,
                                    associated_data: key.associated_data,
                                },
                            ));
                        }
                    }
                }
                VaultRequestPayload::Bootstrap(req) => {
                    let Some(encrypted_key) = req.encrypted_key else {
                        let _ = self
                            .bi_mut()
                            .send(Err(Status::invalid_argument(
                                "Missing bootstrap encrypted key",
                            )))
                            .await;
                        return None;
                    };
                    return Some(vault_gate::Inbound::HandleBootstrapEncryptedKey(
                        HandleBootstrapEncryptedKey {
                            nonce: encrypted_key.nonce,
                            ciphertext: encrypted_key.ciphertext,
                            associated_data: encrypted_key.associated_data,
                        },
                    ));
                }
            }
        }
    }
}

#[async_trait]
impl Sender<Result<vault_gate::Outbound, vault_gate::Error>> for AuthTransportAdapter<'_> {
    async fn send(
        &mut self,
        item: Result<vault_gate::Outbound, vault_gate::Error>,
    ) -> Result<(), TransportError> {
        let outbound = match item {
            Ok(outbound) => outbound,
            Err(err) => {
                warn!(?err, "vault gate produced transport-level error");
                return self
                    .bi_mut()
                    .send(Err(Status::internal(err.to_string())))
                    .await;
            }
        };

        let payload = match outbound {
            vault_gate::Outbound::HandleHandshake(Ok(response)) => {
                wrap_unseal_response(UnsealResponsePayload::Start(
                    proto_unseal::UnsealStartResponse {
                        server_pubkey: response.server_pubkey.as_bytes().to_vec(),
                    },
                ))
            }
            vault_gate::Outbound::HandleHandshake(Err(err)) => {
                warn!(?err, "handshake failed");
                return self
                    .bi_mut()
                    .send(Err(Status::internal("Failed to start unseal flow")))
                    .await;
            }
            vault_gate::Outbound::HandleUnsealEncryptedKey(result) => {
                let proto_result = match result {
                    Ok(()) => ProtoUnsealResult::Success,
                    Err(vault_gate::Error::InvalidKey) => ProtoUnsealResult::InvalidKey,
                    Err(err) => {
                        warn!(?err, "unseal failed");
                        return self
                            .bi_mut()
                            .send(Err(Status::internal("Failed to unseal vault")))
                            .await;
                    }
                };
                wrap_unseal_response(UnsealResponsePayload::Result(proto_result.into()))
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
                        return self
                            .bi_mut()
                            .send(Err(Status::internal("Failed to bootstrap vault")))
                            .await;
                    }
                };
                wrap_bootstrap_response(proto_result)
            }
        };

        self.send_response_payload(payload).await
    }
}

impl Bi<vault_gate::Inbound, Result<vault_gate::Outbound, vault_gate::Error>>
    for AuthTransportAdapter<'_>
{
}
