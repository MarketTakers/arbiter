use arbiter_proto::{
    proto::{
        evm::{
            EvmError as ProtoEvmError, WalletCreateResponse, WalletEntry, WalletList,
            WalletListResponse, wallet_create_response::Result as WalletCreateResult,
            wallet_list_response::Result as WalletListResult,
        },
        user_agent::{
            AuthChallenge as ProtoAuthChallenge,
            AuthChallengeRequest as ProtoAuthChallengeRequest,
            AuthChallengeSolution as ProtoAuthChallengeSolution, AuthOk as ProtoAuthOk,
            BootstrapEncryptedKey as ProtoBootstrapEncryptedKey,
            BootstrapResult as ProtoBootstrapResult, ClientConnectionCancel,
            ClientConnectionRequest, ClientConnectionResponse, KeyType as ProtoKeyType,
            UnsealEncryptedKey as ProtoUnsealEncryptedKey, UnsealResult as ProtoUnsealResult,
            UnsealStart, UnsealStartResponse, UserAgentRequest, UserAgentResponse,
            VaultState as ProtoVaultState,
            user_agent_request::Payload as UserAgentRequestPayload,
            user_agent_response::Payload as UserAgentResponsePayload,
        },
    },
    transport::{Bi, Error as TransportError},
};
use async_trait::async_trait;
use futures::StreamExt as _;
use tokio::sync::mpsc;
use tonic::{Status, Streaming};

use crate::actors::user_agent::{
    self, AuthPublicKey, BootstrapError, Request as DomainRequest, Response as DomainResponse,
    TransportResponseError, UnsealError, VaultState,
};

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
            Some(UserAgentRequestPayload::AuthChallengeRequest(
                ProtoAuthChallengeRequest {
                    pubkey,
                    bootstrap_token,
                    key_type,
                },
            )) => Ok(DomainRequest::AuthChallengeRequest {
                pubkey: parse_auth_pubkey(key_type, pubkey)?,
                bootstrap_token,
            }),
            Some(UserAgentRequestPayload::AuthChallengeSolution(
                ProtoAuthChallengeSolution { signature },
            )) => Ok(DomainRequest::AuthChallengeSolution { signature }),
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
            Some(UserAgentRequestPayload::BootstrapEncryptedKey(
                ProtoBootstrapEncryptedKey {
                    nonce,
                    ciphertext,
                    associated_data,
                },
            )) => Ok(DomainRequest::BootstrapEncryptedKey {
                nonce,
                ciphertext,
                associated_data,
            }),
            Some(UserAgentRequestPayload::QueryVaultState(_)) => {
                Ok(DomainRequest::QueryVaultState)
            }
            Some(UserAgentRequestPayload::EvmWalletCreate(_)) => Ok(DomainRequest::EvmWalletCreate),
            Some(UserAgentRequestPayload::EvmWalletList(_)) => Ok(DomainRequest::EvmWalletList),
            Some(UserAgentRequestPayload::ClientConnectionResponse(
                ClientConnectionResponse { approved },
            )) => Ok(DomainRequest::ClientConnectionResponse { approved }),
            Some(_) => Err(Status::invalid_argument(
                "Unexpected user-agent request payload",
            )),
            None => Err(Status::invalid_argument("Missing user-agent request payload")),
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
            TransportResponseError::StateTransitionFailed => Status::internal("State machine error"),
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
