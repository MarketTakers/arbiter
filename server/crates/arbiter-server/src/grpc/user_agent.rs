use arbiter_proto::{
    proto::{
        self,
        evm::{
            EtherTransferSettings as ProtoEtherTransferSettings, EvmError as ProtoEvmError,
            EvmGrantCreateRequest, EvmGrantCreateResponse, EvmGrantDeleteRequest,
            EvmGrantDeleteResponse, EvmGrantList, EvmGrantListResponse, GrantEntry,
            SharedSettings as ProtoSharedSettings, SpecificGrant as ProtoSpecificGrant,
            TokenTransferSettings as ProtoTokenTransferSettings,
            VolumeRateLimit as ProtoVolumeRateLimit, WalletCreateResponse, WalletEntry, WalletList,
            WalletListResponse, evm_grant_create_response::Result as EvmGrantCreateResult,
            evm_grant_delete_response::Result as EvmGrantDeleteResult,
            evm_grant_list_response::Result as EvmGrantListResult,
            specific_grant::Grant as ProtoSpecificGrantType,
            wallet_create_response::Result as WalletCreateResult,
            wallet_list_response::Result as WalletListResult,
        },
        user_agent::{
            AuthChallenge as ProtoAuthChallenge, AuthChallengeRequest as ProtoAuthChallengeRequest,
            AuthChallengeSolution as ProtoAuthChallengeSolution, AuthOk as ProtoAuthOk,
            BootstrapEncryptedKey as ProtoBootstrapEncryptedKey,
            BootstrapResult as ProtoBootstrapResult, ClientConnectionCancel,
            ClientConnectionRequest, ClientConnectionResponse, KeyType as ProtoKeyType,
            UnsealEncryptedKey as ProtoUnsealEncryptedKey, UnsealResult as ProtoUnsealResult,
            UnsealStart, UnsealStartResponse, UserAgentRequest, UserAgentResponse,
            VaultState as ProtoVaultState, user_agent_request::Payload as UserAgentRequestPayload,
            user_agent_response::Payload as UserAgentResponsePayload,
        },
    },
    transport::{Bi, Error as TransportError},
};
use async_trait::async_trait;
use futures::StreamExt as _;
use prost_types::Timestamp;
use tokio::sync::mpsc;
use tonic::{Status, Streaming};

use crate::{
    actors::user_agent::{
        self, AuthPublicKey, BootstrapError, Request as DomainRequest, Response as DomainResponse,
        TransportResponseError, UnsealError, VaultState,
    },
    evm::{
        policies::{Grant, SpecificGrant},
        policies::{
            SharedGrantSettings, TransactionRateLimit, VolumeRateLimit, ether_transfer,
            token_transfers,
        },
    },
};
use alloy::primitives::{Address, U256};
use chrono::{DateTime, TimeZone, Utc};

pub struct GrpcTransport {
    sender: mpsc::Sender<Result<UserAgentResponse, Status>>,
    receiver: Streaming<UserAgentRequest>,
}

impl GrpcTransport {
    pub fn new(
        sender: mpsc::Sender<Result<UserAgentResponse, Status>>,
        receiver: Streaming<UserAgentRequest>,
    ) -> Self {
        Self { sender, receiver }
    }

    fn request_to_domain(request: UserAgentRequest) -> Result<DomainRequest, Status> {
        match request.payload {
            Some(UserAgentRequestPayload::AuthChallengeRequest(ProtoAuthChallengeRequest {
                pubkey,
                bootstrap_token,
                key_type,
            })) => Ok(DomainRequest::AuthChallengeRequest {
                pubkey: parse_auth_pubkey(key_type, pubkey)?,
                bootstrap_token,
            }),
            Some(UserAgentRequestPayload::AuthChallengeSolution(ProtoAuthChallengeSolution {
                signature,
            })) => Ok(DomainRequest::AuthChallengeSolution { signature }),
            Some(UserAgentRequestPayload::UnsealStart(UnsealStart { client_pubkey })) => {
                let client_pubkey: [u8; 32] = client_pubkey
                    .as_slice()
                    .try_into()
                    .map_err(|_| Status::invalid_argument("client_pubkey must be 32 bytes"))?;
                Ok(DomainRequest::UnsealStart {
                    client_pubkey: x25519_dalek::PublicKey::from(client_pubkey),
                })
            }
            Some(UserAgentRequestPayload::UnsealEncryptedKey(ProtoUnsealEncryptedKey {
                nonce,
                ciphertext,
                associated_data,
            })) => Ok(DomainRequest::UnsealEncryptedKey {
                nonce,
                ciphertext,
                associated_data,
            }),
            Some(UserAgentRequestPayload::BootstrapEncryptedKey(ProtoBootstrapEncryptedKey {
                nonce,
                ciphertext,
                associated_data,
            })) => Ok(DomainRequest::BootstrapEncryptedKey {
                nonce,
                ciphertext,
                associated_data,
            }),
            Some(UserAgentRequestPayload::QueryVaultState(_)) => Ok(DomainRequest::QueryVaultState),
            Some(UserAgentRequestPayload::EvmWalletCreate(_)) => Ok(DomainRequest::EvmWalletCreate),
            Some(UserAgentRequestPayload::EvmWalletList(_)) => Ok(DomainRequest::EvmWalletList),
            Some(UserAgentRequestPayload::ClientConnectionResponse(ClientConnectionResponse {
                approved,
            })) => Ok(DomainRequest::ClientConnectionResponse { approved }),

            Some(UserAgentRequestPayload::EvmGrantList(_)) => Ok(DomainRequest::ListGrants),
            Some(UserAgentRequestPayload::EvmGrantCreate(EvmGrantCreateRequest {
                client_id,
                shared,
                specific,
            })) => {
                let shared = parse_shared_settings(client_id, shared)?;
                let specific = parse_specific_grant(specific)?;
                Ok(DomainRequest::EvmGrantCreate {
                    client_id,
                    shared,
                    specific,
                })
            }
            Some(UserAgentRequestPayload::EvmGrantDelete(EvmGrantDeleteRequest { grant_id })) => {
                Ok(DomainRequest::EvmGrantDelete { grant_id })
            }
            None => Err(Status::invalid_argument(
                "Missing user-agent request payload",
            )),
        }
    }

    fn response_to_proto(response: DomainResponse) -> UserAgentResponse {
        let payload = match response {
            DomainResponse::AuthChallenge { nonce } => {
                UserAgentResponsePayload::AuthChallenge(ProtoAuthChallenge {
                    pubkey: Vec::new(),
                    nonce,
                })
            }
            DomainResponse::AuthOk => UserAgentResponsePayload::AuthOk(ProtoAuthOk {}),
            DomainResponse::UnsealStartResponse { server_pubkey } => {
                UserAgentResponsePayload::UnsealStartResponse(UnsealStartResponse {
                    server_pubkey: server_pubkey.as_bytes().to_vec(),
                })
            }
            DomainResponse::UnsealResult(result) => UserAgentResponsePayload::UnsealResult(
                match result {
                    Ok(()) => ProtoUnsealResult::Success,
                    Err(UnsealError::InvalidKey) => ProtoUnsealResult::InvalidKey,
                    Err(UnsealError::Unbootstrapped) => ProtoUnsealResult::Unbootstrapped,
                }
                .into(),
            ),
            DomainResponse::BootstrapResult(result) => UserAgentResponsePayload::BootstrapResult(
                match result {
                    Ok(()) => ProtoBootstrapResult::Success,
                    Err(BootstrapError::AlreadyBootstrapped) => {
                        ProtoBootstrapResult::AlreadyBootstrapped
                    }
                    Err(BootstrapError::InvalidKey) => ProtoBootstrapResult::InvalidKey,
                }
                .into(),
            ),
            DomainResponse::VaultState(state) => UserAgentResponsePayload::VaultState(
                match state {
                    VaultState::Unbootstrapped => ProtoVaultState::Unbootstrapped,
                    VaultState::Sealed => ProtoVaultState::Sealed,
                    VaultState::Unsealed => ProtoVaultState::Unsealed,
                }
                .into(),
            ),
            DomainResponse::ClientConnectionRequest { pubkey } => {
                UserAgentResponsePayload::ClientConnectionRequest(ClientConnectionRequest {
                    pubkey: pubkey.to_bytes().to_vec(),
                })
            }
            DomainResponse::ClientConnectionCancel => {
                UserAgentResponsePayload::ClientConnectionCancel(ClientConnectionCancel {})
            }
            DomainResponse::EvmWalletCreate(result) => {
                UserAgentResponsePayload::EvmWalletCreate(WalletCreateResponse {
                    result: Some(match result {
                        Ok(()) => WalletCreateResult::Wallet(WalletEntry {
                            address: Vec::new(),
                        }),
                        Err(_) => WalletCreateResult::Error(ProtoEvmError::Internal.into()),
                    }),
                })
            }
            DomainResponse::EvmWalletList(wallets) => {
                UserAgentResponsePayload::EvmWalletList(WalletListResponse {
                    result: Some(WalletListResult::Wallets(WalletList {
                        wallets: wallets
                            .into_iter()
                            .map(|addr| WalletEntry {
                                address: addr.as_slice().to_vec(),
                            })
                            .collect(),
                    })),
                })
            }
            DomainResponse::ListGrants(grants) => {
                UserAgentResponsePayload::EvmGrantList(EvmGrantListResponse {
                    result: Some(EvmGrantListResult::Grants(EvmGrantList {
                        grants: grants.into_iter().map(grant_to_proto).collect(),
                    })),
                })
            }
            DomainResponse::EvmGrantCreate(result) => {
                UserAgentResponsePayload::EvmGrantCreate(EvmGrantCreateResponse {
                    result: Some(match result {
                        Ok(grant_id) => EvmGrantCreateResult::GrantId(grant_id),
                        Err(_) => EvmGrantCreateResult::Error(ProtoEvmError::Internal.into()),
                    }),
                })
            }
            DomainResponse::EvmGrantDelete(result) => {
                UserAgentResponsePayload::EvmGrantDelete(EvmGrantDeleteResponse {
                    result: Some(match result {
                        Ok(()) => EvmGrantDeleteResult::Ok(()),
                        Err(_) => EvmGrantDeleteResult::Error(ProtoEvmError::Internal.into()),
                    }),
                })
            }
        };

        UserAgentResponse {
            payload: Some(payload),
        }
    }

    fn error_to_status(value: TransportResponseError) -> Status {
        match value {
            TransportResponseError::UnexpectedRequestPayload => {
                Status::invalid_argument("Expected message with payload")
            }
            TransportResponseError::InvalidStateForUnsealEncryptedKey => {
                Status::failed_precondition("Invalid state for unseal encrypted key")
            }
            TransportResponseError::InvalidClientPubkeyLength => {
                Status::invalid_argument("client_pubkey must be 32 bytes")
            }
            TransportResponseError::StateTransitionFailed => {
                Status::internal("State machine error")
            }
            TransportResponseError::KeyHolderActorUnreachable => {
                Status::internal("Vault is not available")
            }
            TransportResponseError::Auth(ref err) => auth_error_status(err),
            TransportResponseError::ConnectionRegistrationFailed => {
                Status::internal("Failed registering connection")
            }
        }
    }
}

#[async_trait]
impl Bi<DomainRequest, Result<DomainResponse, TransportResponseError>> for GrpcTransport {
    async fn send(
        &mut self,
        item: Result<DomainResponse, TransportResponseError>,
    ) -> Result<(), TransportError> {
        let outbound = match item {
            Ok(message) => Ok(Self::response_to_proto(message)),
            Err(err) => Err(Self::error_to_status(err)),
        };

        self.sender
            .send(outbound)
            .await
            .map_err(|_| TransportError::ChannelClosed)
    }

    async fn recv(&mut self) -> Option<DomainRequest> {
        match self.receiver.next().await {
            Some(Ok(item)) => match Self::request_to_domain(item) {
                Ok(request) => Some(request),
                Err(status) => {
                    let _ = self.sender.send(Err(status)).await;
                    None
                }
            },
            Some(Err(error)) => {
                tracing::error!(error = ?error, "grpc user-agent recv failed; closing stream");
                None
            }
            None => None,
        }
    }
}

fn grant_to_proto(grant: Grant<SpecificGrant>) -> proto::evm::GrantEntry {
    GrantEntry {
        id: grant.id,
        specific: Some(match grant.settings {
            SpecificGrant::EtherTransfer(settings) => ProtoSpecificGrant {
                grant: Some(ProtoSpecificGrantType::EtherTransfer(
                    ProtoEtherTransferSettings {
                        targets: settings
                            .target
                            .into_iter()
                            .map(|addr| addr.as_slice().to_vec())
                            .collect(),
                        limit: Some(proto::evm::VolumeRateLimit {
                            max_volume: settings.limit.max_volume.to_be_bytes_vec(),
                            window_secs: settings.limit.window.num_seconds(),
                        }),
                    },
                )),
            },
            SpecificGrant::TokenTransfer(settings) => ProtoSpecificGrant {
                grant: Some(ProtoSpecificGrantType::TokenTransfer(
                    ProtoTokenTransferSettings {
                        token_contract: settings.token_contract.as_slice().to_vec(),
                        target: settings.target.map(|addr| addr.as_slice().to_vec()),
                        volume_limits: settings
                            .volume_limits
                            .into_iter()
                            .map(|vrl| proto::evm::VolumeRateLimit {
                                max_volume: vrl.max_volume.to_be_bytes_vec(),
                                window_secs: vrl.window.num_seconds(),
                            })
                            .collect(),
                    },
                )),
            },
        }),
        client_id: grant.shared.client_id,
        shared: Some(proto::evm::SharedSettings {
            wallet_id: grant.shared.wallet_id,
            chain_id: grant.shared.chain,
            valid_from: grant.shared.valid_from.map(|dt| Timestamp {
                seconds: dt.timestamp(),
                nanos: 0,
            }),
            valid_until: grant.shared.valid_until.map(|dt| Timestamp {
                seconds: dt.timestamp(),
                nanos: 0,
            }),
            max_gas_fee_per_gas: grant
                .shared
                .max_gas_fee_per_gas
                .map(|fee| fee.to_be_bytes_vec()),
            max_priority_fee_per_gas: grant
                .shared
                .max_priority_fee_per_gas
                .map(|fee| fee.to_be_bytes_vec()),
            rate_limit: grant
                .shared
                .rate_limit
                .map(|limit| proto::evm::TransactionRateLimit {
                    count: limit.count,
                    window_secs: limit.window.num_seconds(),
                }),
        }),
    }
}

fn parse_volume_rate_limit(vrl: ProtoVolumeRateLimit) -> Result<VolumeRateLimit, Status> {
    Ok(VolumeRateLimit {
        max_volume: U256::from_be_slice(&vrl.max_volume),
        window: chrono::Duration::seconds(vrl.window_secs),
    })
}

fn parse_shared_settings(
    client_id: i32,
    proto: Option<ProtoSharedSettings>,
) -> Result<SharedGrantSettings, Status> {
    let s = proto.ok_or_else(|| Status::invalid_argument("missing shared settings"))?;
    let parse_u256 = |b: Vec<u8>| -> Result<U256, Status> {
        if b.is_empty() {
            Err(Status::invalid_argument("U256 bytes must not be empty"))
        } else {
            Ok(U256::from_be_slice(&b))
        }
    };
    let parse_ts = |ts: prost_types::Timestamp| -> Result<DateTime<Utc>, Status> {
        Utc.timestamp_opt(ts.seconds, ts.nanos as u32)
            .single()
            .ok_or_else(|| Status::invalid_argument("invalid timestamp"))
    };
    Ok(SharedGrantSettings {
        wallet_id: s.wallet_id,
        client_id,
        chain: s.chain_id,
        valid_from: s.valid_from.map(parse_ts).transpose()?,
        valid_until: s.valid_until.map(parse_ts).transpose()?,
        max_gas_fee_per_gas: s.max_gas_fee_per_gas.map(parse_u256).transpose()?,
        max_priority_fee_per_gas: s.max_priority_fee_per_gas.map(parse_u256).transpose()?,
        rate_limit: s.rate_limit.map(|rl| TransactionRateLimit {
            count: rl.count,
            window: chrono::Duration::seconds(rl.window_secs),
        }),
    })
}

fn parse_specific_grant(proto: Option<proto::evm::SpecificGrant>) -> Result<SpecificGrant, Status> {
    use proto::evm::specific_grant::Grant as ProtoGrant;
    let g = proto
        .and_then(|sg| sg.grant)
        .ok_or_else(|| Status::invalid_argument("missing specific grant"))?;
    match g {
        ProtoGrant::EtherTransfer(s) => {
            let limit = parse_volume_rate_limit(
                s.limit
                    .ok_or_else(|| Status::invalid_argument("missing ether transfer limit"))?,
            )?;
            let target = s
                .targets
                .into_iter()
                .map(|b| {
                    if b.len() == 20 {
                        Ok(Address::from_slice(&b))
                    } else {
                        Err(Status::invalid_argument(
                            "ether transfer target must be 20 bytes",
                        ))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(SpecificGrant::EtherTransfer(ether_transfer::Settings {
                target,
                limit,
            }))
        }
        ProtoGrant::TokenTransfer(s) => {
            if s.token_contract.len() != 20 {
                return Err(Status::invalid_argument("token_contract must be 20 bytes"));
            }
            let target = s
                .target
                .map(|b| {
                    if b.len() == 20 {
                        Ok(Address::from_slice(&b))
                    } else {
                        Err(Status::invalid_argument(
                            "token transfer target must be 20 bytes",
                        ))
                    }
                })
                .transpose()?;
            let volume_limits = s
                .volume_limits
                .into_iter()
                .map(parse_volume_rate_limit)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(SpecificGrant::TokenTransfer(token_transfers::Settings {
                token_contract: Address::from_slice(&s.token_contract),
                target,
                volume_limits,
            }))
        }
    }
}

fn parse_auth_pubkey(key_type: i32, pubkey: Vec<u8>) -> Result<AuthPublicKey, Status> {
    match ProtoKeyType::try_from(key_type).unwrap_or(ProtoKeyType::Unspecified) {
        ProtoKeyType::Unspecified | ProtoKeyType::Ed25519 => {
            let bytes: [u8; 32] = pubkey
                .as_slice()
                .try_into()
                .map_err(|_| Status::invalid_argument("invalid Ed25519 public key length"))?;
            let key = ed25519_dalek::VerifyingKey::from_bytes(&bytes)
                .map_err(|_| Status::invalid_argument("invalid Ed25519 public key encoding"))?;
            Ok(AuthPublicKey::Ed25519(key))
        }
        ProtoKeyType::EcdsaSecp256k1 => {
            let key = k256::ecdsa::VerifyingKey::from_sec1_bytes(&pubkey)
                .map_err(|_| Status::invalid_argument("invalid secp256k1 public key encoding"))?;
            Ok(AuthPublicKey::EcdsaSecp256k1(key))
        }
        ProtoKeyType::Rsa => {
            use rsa::pkcs8::DecodePublicKey as _;

            let key = rsa::RsaPublicKey::from_public_key_der(&pubkey)
                .map_err(|_| Status::invalid_argument("invalid RSA public key encoding"))?;
            Ok(AuthPublicKey::Rsa(key))
        }
    }
}

fn auth_error_status(value: &user_agent::auth::Error) -> Status {
    use user_agent::auth::Error;

    match value {
        Error::UnexpectedMessagePayload | Error::InvalidClientPubkeyLength => {
            Status::invalid_argument(value.to_string())
        }
        Error::InvalidAuthPubkeyEncoding => {
            Status::invalid_argument("Failed to convert pubkey to VerifyingKey")
        }
        Error::PublicKeyNotRegistered | Error::InvalidChallengeSolution => {
            Status::unauthenticated(value.to_string())
        }
        Error::InvalidBootstrapToken => Status::invalid_argument("Invalid bootstrap token"),
        Error::Transport => Status::internal("Transport error"),
        Error::BootstrapperActorUnreachable => {
            Status::internal("Bootstrap token consumption failed")
        }
        Error::DatabasePoolUnavailable => Status::internal("Database pool error"),
        Error::DatabaseOperationFailed => Status::internal("Database error"),
    }
}
