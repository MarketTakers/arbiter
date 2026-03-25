use tokio::sync::mpsc;

use arbiter_proto::{
    proto::{
        client::ClientInfo as ProtoClientMetadata,
        evm::{
            EvmError as ProtoEvmError, EvmGrantCreateRequest, EvmGrantCreateResponse,
            EvmGrantDeleteRequest, EvmGrantDeleteResponse, EvmGrantList, EvmGrantListResponse,
            GrantEntry, WalletCreateResponse, WalletEntry, WalletList, WalletListResponse,
            evm_grant_create_response::Result as EvmGrantCreateResult,
            evm_grant_delete_response::Result as EvmGrantDeleteResult,
            evm_grant_list_response::Result as EvmGrantListResult,
            wallet_create_response::Result as WalletCreateResult,
            wallet_list_response::Result as WalletListResult,
        },
        user_agent::{
            BootstrapEncryptedKey as ProtoBootstrapEncryptedKey,
            BootstrapResult as ProtoBootstrapResult,
            SdkClientConnectionCancel as ProtoSdkClientConnectionCancel,
            SdkClientConnectionRequest as ProtoSdkClientConnectionRequest,
            SdkClientEntry as ProtoSdkClientEntry, SdkClientError as ProtoSdkClientError,
            SdkClientList as ProtoSdkClientList,
            SdkClientListResponse as ProtoSdkClientListResponse,
            UnsealEncryptedKey as ProtoUnsealEncryptedKey, UnsealResult as ProtoUnsealResult,
            UnsealStart, UserAgentRequest, UserAgentResponse, VaultState as ProtoVaultState,
            sdk_client_list_response::Result as ProtoSdkClientListResult,
            user_agent_request::Payload as UserAgentRequestPayload,
            user_agent_response::Payload as UserAgentResponsePayload,
        },
    },
    transport::{Error as TransportError, Receiver, Sender, grpc::GrpcBi},
};
use async_trait::async_trait;
use kameo::{
    actor::{ActorRef, Spawn as _},
    error::SendError,
};
use tonic::Status;
use tracing::{error, info, warn};

use crate::{
    actors::{
        keyholder::KeyHolderState,
        user_agent::{
            OutOfBand, UserAgentConnection, UserAgentSession,
            session::{
                BootstrapError, HandleBootstrapEncryptedKey, HandleEvmWalletCreate,
                HandleEvmWalletList, HandleGrantCreate, HandleGrantDelete, HandleGrantList,
                HandleNewClientApprove, HandleQueryVaultState, HandleSdkClientList,
                HandleUnsealEncryptedKey, HandleUnsealRequest, UnsealError,
            },
        },
    },
    grpc::{Convert, TryConvert, request_tracker::RequestTracker},
};
mod auth;
mod inbound;
mod outbound;

pub struct OutOfBandAdapter(mpsc::Sender<OutOfBand>);

#[async_trait]
impl Sender<OutOfBand> for OutOfBandAdapter {
    async fn send(&mut self, item: OutOfBand) -> Result<(), TransportError> {
        self.0.send(item).await.map_err(|e| {
            warn!(error = ?e, "Failed to send out-of-band message");
            TransportError::ChannelClosed
        })
    }
}

async fn dispatch_loop(
    mut bi: GrpcBi<UserAgentRequest, UserAgentResponse>,
    actor: ActorRef<UserAgentSession>,
    mut receiver: mpsc::Receiver<OutOfBand>,
    mut request_tracker: RequestTracker,
) {
    loop {
        tokio::select! {
            oob = receiver.recv() => {
                let Some(oob) = oob else {
                    warn!("Out-of-band message channel closed");
                    return;
                };

                let payload = match oob {
                    OutOfBand::ClientConnectionRequest { profile } => {
                        UserAgentResponsePayload::SdkClientConnectionRequest(ProtoSdkClientConnectionRequest {
                            pubkey: profile.pubkey.to_bytes().to_vec(),
                            info: Some(ProtoClientMetadata {
                                name: profile.metadata.name,
                                description: profile.metadata.description,
                                version: profile.metadata.version,
                            }),
                        })
                    }
                    OutOfBand::ClientConnectionCancel { pubkey } => {
                        UserAgentResponsePayload::SdkClientConnectionCancel(ProtoSdkClientConnectionCancel {
                            pubkey: pubkey.to_bytes().to_vec(),
                        })
                    }
                };

                if bi.send(Ok(UserAgentResponse { id: None, payload: Some(payload) })).await.is_err() {
                    return;
                }
            }

            message = bi.recv() => {
                let Some(message) = message else { return; };

                let conn = match message {
                    Ok(conn) => conn,
                    Err(err) => {
                        warn!(error = ?err, "Failed to receive user agent request");
                        return;
                    }
                };

                let request_id = match request_tracker.request(conn.id) {
                    Ok(id) => id,
                    Err(err) => {
                        let _ = bi.send(Err(err)).await;
                        return;
                    }
                };

                let Some(payload) = conn.payload else {
                    let _ = bi.send(Err(Status::invalid_argument("Missing user-agent request payload"))).await;
                    return;
                };

                match dispatch_inner(&actor, payload).await {
                    Ok(Some(response)) => {
                        if bi.send(Ok(UserAgentResponse {
                            id: Some(request_id),
                            payload: Some(response),
                        })).await.is_err() {
                            return;
                        }
                    }
                    Ok(None) => {}
                    Err(status) => {
                         error!(?status, "Failed to process user agent request");
                        let _ = bi.send(Err(status)).await;
                        return;
                    }
                }
            }
        }
    }
}

async fn dispatch_inner(
    actor: &ActorRef<UserAgentSession>,
    payload: UserAgentRequestPayload,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let response = match payload {
        UserAgentRequestPayload::UnsealStart(UnsealStart { client_pubkey }) => {
            let client_pubkey = <[u8; 32]>::try_from(client_pubkey)
                .map(x25519_dalek::PublicKey::from)
                .map_err(|_| Status::invalid_argument("Invalid X25519 public key"))?;

            let response = actor
                .ask(HandleUnsealRequest { client_pubkey })
                .await
                .map_err(|err| {
                    warn!(error = ?err, "Failed to handle unseal start request");
                    Status::internal("Failed to start unseal flow")
                })?;

            UserAgentResponsePayload::UnsealStartResponse(
                arbiter_proto::proto::user_agent::UnsealStartResponse {
                    server_pubkey: response.server_pubkey.as_bytes().to_vec(),
                },
            )
        }

        UserAgentRequestPayload::UnsealEncryptedKey(ProtoUnsealEncryptedKey {
            nonce,
            ciphertext,
            associated_data,
        }) => {
            let result = match actor
                .ask(HandleUnsealEncryptedKey {
                    nonce,
                    ciphertext,
                    associated_data,
                })
                .await
            {
                Ok(()) => ProtoUnsealResult::Success,
                Err(SendError::HandlerError(UnsealError::InvalidKey)) => {
                    ProtoUnsealResult::InvalidKey
                }
                Err(err) => {
                    warn!(error = ?err, "Failed to handle unseal request");
                    return Err(Status::internal("Failed to unseal vault"));
                }
            };
            UserAgentResponsePayload::UnsealResult(result.into())
        }

        UserAgentRequestPayload::BootstrapEncryptedKey(ProtoBootstrapEncryptedKey {
            nonce,
            ciphertext,
            associated_data,
        }) => {
            let result = match actor
                .ask(HandleBootstrapEncryptedKey {
                    nonce,
                    ciphertext,
                    associated_data,
                })
                .await
            {
                Ok(()) => ProtoBootstrapResult::Success,
                Err(SendError::HandlerError(BootstrapError::InvalidKey)) => {
                    ProtoBootstrapResult::InvalidKey
                }
                Err(SendError::HandlerError(BootstrapError::AlreadyBootstrapped)) => {
                    ProtoBootstrapResult::AlreadyBootstrapped
                }
                Err(err) => {
                    warn!(error = ?err, "Failed to handle bootstrap request");
                    return Err(Status::internal("Failed to bootstrap vault"));
                }
            };
            UserAgentResponsePayload::BootstrapResult(result.into())
        }

        UserAgentRequestPayload::QueryVaultState(_) => {
            let state = match actor.ask(HandleQueryVaultState {}).await {
                Ok(KeyHolderState::Unbootstrapped) => ProtoVaultState::Unbootstrapped,
                Ok(KeyHolderState::Sealed) => ProtoVaultState::Sealed,
                Ok(KeyHolderState::Unsealed) => ProtoVaultState::Unsealed,
                Err(err) => {
                    warn!(error = ?err, "Failed to query vault state");
                    ProtoVaultState::Error
                }
            };
            UserAgentResponsePayload::VaultState(state.into())
        }

        UserAgentRequestPayload::EvmWalletCreate(_) => {
            let result = match actor.ask(HandleEvmWalletCreate {}).await {
                Ok((wallet_id, address)) => WalletCreateResult::Wallet(WalletEntry {
                    id: wallet_id,
                    address: address.to_vec(),
                }),
                Err(err) => {
                    warn!(error = ?err, "Failed to create EVM wallet");
                    WalletCreateResult::Error(ProtoEvmError::Internal.into())
                }
            };
            UserAgentResponsePayload::EvmWalletCreate(WalletCreateResponse {
                result: Some(result),
            })
        }

        UserAgentRequestPayload::EvmWalletList(_) => {
            let result = match actor.ask(HandleEvmWalletList {}).await {
                Ok(wallets) => WalletListResult::Wallets(WalletList {
                    wallets: wallets
                        .into_iter()
                        .map(|w| WalletEntry {
                            address: w.to_vec(),
                            id: todo!(),
                        })
                        .collect(),
                }),
                Err(err) => {
                    warn!(error = ?err, "Failed to list EVM wallets");
                    WalletListResult::Error(ProtoEvmError::Internal.into())
                }
            };
            UserAgentResponsePayload::EvmWalletList(WalletListResponse {
                result: Some(result),
            })
        }

        UserAgentRequestPayload::EvmGrantList(_) => {
            let result = match actor.ask(HandleGrantList {}).await {
                Ok(grants) => EvmGrantListResult::Grants(EvmGrantList {
                    grants: grants
                        .into_iter()
                        .map(|grant| GrantEntry {
                            id: grant.id,
                            wallet_access_id: grant.shared.wallet_access_id,
                            shared: Some(grant.shared.convert()),
                            specific: Some(grant.settings.convert()),
                        })
                        .collect(),
                }),
                Err(err) => {
                    warn!(error = ?err, "Failed to list EVM grants");
                    EvmGrantListResult::Error(ProtoEvmError::Internal.into())
                }
            };
            UserAgentResponsePayload::EvmGrantList(EvmGrantListResponse {
                result: Some(result),
            })
        }

        UserAgentRequestPayload::EvmGrantCreate(EvmGrantCreateRequest { shared, specific }) => {
            let basic = shared
                .ok_or_else(|| Status::invalid_argument("Missing shared grant settings"))?
                .try_convert()?;
            let grant = specific
                .ok_or_else(|| Status::invalid_argument("Missing specific grant settings"))?
                .try_convert()?;

            let result = match actor.ask(HandleGrantCreate { basic, grant }).await {
                Ok(grant_id) => EvmGrantCreateResult::GrantId(grant_id),
                Err(err) => {
                    warn!(error = ?err, "Failed to create EVM grant");
                    EvmGrantCreateResult::Error(ProtoEvmError::Internal.into())
                }
            };
            UserAgentResponsePayload::EvmGrantCreate(EvmGrantCreateResponse {
                result: Some(result),
            })
        }

        UserAgentRequestPayload::EvmGrantDelete(EvmGrantDeleteRequest { grant_id }) => {
            let result = match actor.ask(HandleGrantDelete { grant_id }).await {
                Ok(()) => EvmGrantDeleteResult::Ok(()),
                Err(err) => {
                    warn!(error = ?err, "Failed to delete EVM grant");
                    EvmGrantDeleteResult::Error(ProtoEvmError::Internal.into())
                }
            };
            UserAgentResponsePayload::EvmGrantDelete(EvmGrantDeleteResponse {
                result: Some(result),
            })
        }

        UserAgentRequestPayload::SdkClientConnectionResponse(resp) => {
            let pubkey_bytes = <[u8; 32]>::try_from(resp.pubkey)
                .map_err(|_| Status::invalid_argument("Invalid Ed25519 public key length"))?;
            let pubkey = ed25519_dalek::VerifyingKey::from_bytes(&pubkey_bytes)
                .map_err(|_| Status::invalid_argument("Invalid Ed25519 public key"))?;

            actor
                .ask(HandleNewClientApprove {
                    approved: resp.approved,
                    pubkey,
                })
                .await
                .map_err(|err| {
                    warn!(?err, "Failed to process client connection response");
                    Status::internal("Failed to process response")
                })?;

            return Ok(None);
        }

        UserAgentRequestPayload::SdkClientRevoke(_) => todo!(),

        UserAgentRequestPayload::SdkClientList(_) => {
            let result = match actor.ask(HandleSdkClientList {}).await {
                Ok(clients) => ProtoSdkClientListResult::Clients(ProtoSdkClientList {
                    clients: clients
                        .into_iter()
                        .map(|(client, metadata)| ProtoSdkClientEntry {
                            id: client.id,
                            pubkey: client.public_key,
                            info: Some(ProtoClientMetadata {
                                name: metadata.name,
                                description: metadata.description,
                                version: metadata.version,
                            }),
                            created_at: client.created_at.0.timestamp() as i32,
                        })
                        .collect(),
                }),
                Err(err) => {
                    warn!(error = ?err, "Failed to list SDK clients");
                    ProtoSdkClientListResult::Error(ProtoSdkClientError::Internal.into())
                }
            };
            UserAgentResponsePayload::SdkClientListResponse(ProtoSdkClientListResponse {
                result: Some(result),
            })
        }

        UserAgentRequestPayload::GrantWalletAccessList(_)
        | UserAgentRequestPayload::RevokeWalletAccessList(_) => todo!(),

        UserAgentRequestPayload::AuthChallengeRequest(..)
        | UserAgentRequestPayload::AuthChallengeSolution(..) => {
            warn!(?payload, "Unsupported post-auth user agent request");
            return Err(Status::invalid_argument("Unsupported user-agent request"));
        }
    };

    Ok(Some(response))
}

pub async fn start(
    mut conn: UserAgentConnection,
    mut bi: GrpcBi<UserAgentRequest, UserAgentResponse>,
) {
    let mut request_tracker = RequestTracker::default();

    let pubkey = match auth::start(&mut conn, &mut bi, &mut request_tracker).await {
        Ok(pubkey) => pubkey,
        Err(e) => {
            warn!(error = ?e, "Authentication failed");
            return;
        }
    };

    let (oob_sender, oob_receiver) = mpsc::channel(16);
    let oob_adapter = OutOfBandAdapter(oob_sender);

    let actor = UserAgentSession::spawn(UserAgentSession::new(conn, Box::new(oob_adapter)));
    let actor_for_cleanup = actor.clone();

    info!(?pubkey, "User authenticated successfully");
    dispatch_loop(bi, actor, oob_receiver, request_tracker).await;
    actor_for_cleanup.kill();
}
