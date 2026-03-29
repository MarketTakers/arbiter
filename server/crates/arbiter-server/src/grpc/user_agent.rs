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
            BootstrapResult as ProtoBootstrapResult, ListWalletAccessResponse,
            SdkClientConnectionCancel as ProtoSdkClientConnectionCancel,
            SdkClientConnectionRequest as ProtoSdkClientConnectionRequest,
            SdkClientEntry as ProtoSdkClientEntry, SdkClientError as ProtoSdkClientError,
            SdkClientGrantWalletAccess, SdkClientList as ProtoSdkClientList,
            SdkClientListResponse as ProtoSdkClientListResponse, SdkClientRevokeWalletAccess,
            UnsealEncryptedKey as ProtoUnsealEncryptedKey,
            UnsealResult as ProtoUnsealResult, UnsealStart, UserAgentRequest, UserAgentResponse,
            VaultState as ProtoVaultState,
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
            session::connection::{
                BootstrapError, HandleBootstrapEncryptedKey, HandleEvmWalletCreate,
                HandleEvmWalletList, HandleGrantCreate, HandleGrantDelete,
                HandleGrantEvmWalletAccess, HandleGrantList, HandleListWalletAccess,
                HandleNewClientApprove, HandleQueryVaultState, HandleRevokeEvmWalletAccess,
                HandleSdkClientList, HandleUnsealEncryptedKey, HandleUnsealRequest, UnsealError,
            },
        },
    },
    db::models::NewEvmWalletAccess,
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
    match payload {
        UserAgentRequestPayload::UnsealStart(req) => handle_unseal_start(actor, req).await,
        UserAgentRequestPayload::UnsealEncryptedKey(req) => {
            handle_unseal_encrypted_key(actor, req).await
        }
        UserAgentRequestPayload::BootstrapEncryptedKey(req) => {
            handle_bootstrap_encrypted_key(actor, req).await
        }
        UserAgentRequestPayload::QueryVaultState(_) => handle_query_vault_state(actor).await,
        UserAgentRequestPayload::EvmWalletCreate(_) => handle_evm_wallet_create(actor).await,
        UserAgentRequestPayload::EvmWalletList(_) => handle_evm_wallet_list(actor).await,
        UserAgentRequestPayload::EvmGrantList(_) => handle_evm_grant_list(actor).await,
        UserAgentRequestPayload::EvmGrantCreate(req) => handle_evm_grant_create(actor, req).await,
        UserAgentRequestPayload::EvmGrantDelete(req) => handle_evm_grant_delete(actor, req).await,
        UserAgentRequestPayload::SdkClientConnectionResponse(resp) => {
            handle_sdk_client_connection_response(actor, resp).await
        }
        UserAgentRequestPayload::SdkClientRevoke(_) => {
            Err(Status::unimplemented("SdkClientRevoke is not yet implemented"))
        }
        UserAgentRequestPayload::SdkClientList(_) => handle_sdk_client_list(actor).await,
        UserAgentRequestPayload::GrantWalletAccess(req) => {
            handle_grant_wallet_access(actor, req).await
        }
        UserAgentRequestPayload::RevokeWalletAccess(req) => {
            handle_revoke_wallet_access(actor, req).await
        }
        UserAgentRequestPayload::ListWalletAccess(_) => handle_list_wallet_access(actor).await,
        UserAgentRequestPayload::AuthChallengeRequest(..)
        | UserAgentRequestPayload::AuthChallengeSolution(..) => {
            warn!(?payload, "Unsupported post-auth user agent request");
            Err(Status::invalid_argument("Unsupported user-agent request"))
        }
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

    Ok(Some(UserAgentResponsePayload::UnsealStartResponse(
        arbiter_proto::proto::user_agent::UnsealStartResponse {
            server_pubkey: response.server_pubkey.as_bytes().to_vec(),
        },
    )))
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
    Ok(Some(UserAgentResponsePayload::UnsealResult(result.into())))
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
    Ok(Some(UserAgentResponsePayload::BootstrapResult(result.into())))
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
    Ok(Some(UserAgentResponsePayload::VaultState(state.into())))
}

async fn handle_evm_wallet_create(
    actor: &ActorRef<UserAgentSession>,
) -> Result<Option<UserAgentResponsePayload>, Status> {
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
    Ok(Some(UserAgentResponsePayload::EvmWalletCreate(
        WalletCreateResponse { result: Some(result) },
    )))
}

async fn handle_evm_wallet_list(
    actor: &ActorRef<UserAgentSession>,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let result = match actor.ask(HandleEvmWalletList {}).await {
        Ok(wallets) => WalletListResult::Wallets(WalletList {
            wallets: wallets
                .into_iter()
                .map(|(id, address)| WalletEntry {
                    address: address.to_vec(),
                    id,
                })
                .collect(),
        }),
        Err(err) => {
            warn!(error = ?err, "Failed to list EVM wallets");
            WalletListResult::Error(ProtoEvmError::Internal.into())
        }
    };
    Ok(Some(UserAgentResponsePayload::EvmWalletList(
        WalletListResponse { result: Some(result) },
    )))
}

async fn handle_evm_grant_list(
    actor: &ActorRef<UserAgentSession>,
) -> Result<Option<UserAgentResponsePayload>, Status> {
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
    Ok(Some(UserAgentResponsePayload::EvmGrantList(
        EvmGrantListResponse { result: Some(result) },
    )))
}

async fn handle_evm_grant_create(
    actor: &ActorRef<UserAgentSession>,
    req: EvmGrantCreateRequest,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let basic = req
        .shared
        .ok_or_else(|| Status::invalid_argument("Missing shared grant settings"))?
        .try_convert()?;
    let grant = req
        .specific
        .ok_or_else(|| Status::invalid_argument("Missing specific grant settings"))?
        .try_convert()?;

    let result = match actor.ask(HandleGrantCreate { basic, grant }).await {
        Ok(grant_id) => EvmGrantCreateResult::GrantId(grant_id),
        Err(err) => {
            warn!(error = ?err, "Failed to create EVM grant");
            EvmGrantCreateResult::Error(ProtoEvmError::Internal.into())
        }
    };
    Ok(Some(UserAgentResponsePayload::EvmGrantCreate(
        EvmGrantCreateResponse { result: Some(result) },
    )))
}

async fn handle_evm_grant_delete(
    actor: &ActorRef<UserAgentSession>,
    req: EvmGrantDeleteRequest,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let result = match actor.ask(HandleGrantDelete { grant_id: req.grant_id }).await {
        Ok(()) => EvmGrantDeleteResult::Ok(()),
        Err(err) => {
            warn!(error = ?err, "Failed to delete EVM grant");
            EvmGrantDeleteResult::Error(ProtoEvmError::Internal.into())
        }
    };
    Ok(Some(UserAgentResponsePayload::EvmGrantDelete(
        EvmGrantDeleteResponse { result: Some(result) },
    )))
}

async fn handle_sdk_client_connection_response(
    actor: &ActorRef<UserAgentSession>,
    resp: arbiter_proto::proto::user_agent::SdkClientConnectionResponse,
) -> Result<Option<UserAgentResponsePayload>, Status> {
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

    Ok(None)
}

async fn handle_sdk_client_list(
    actor: &ActorRef<UserAgentSession>,
) -> Result<Option<UserAgentResponsePayload>, Status> {
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
    Ok(Some(UserAgentResponsePayload::SdkClientListResponse(
        ProtoSdkClientListResponse { result: Some(result) },
    )))
}

async fn handle_grant_wallet_access(
    actor: &ActorRef<UserAgentSession>,
    req: SdkClientGrantWalletAccess,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let entries: Vec<NewEvmWalletAccess> = req.accesses.into_iter().map(|a| a.convert()).collect();
    match actor.ask(HandleGrantEvmWalletAccess { entries }).await {
        Ok(()) => {
            info!("Successfully granted wallet access");
            Ok(None)
        }
        Err(err) => {
            warn!(error = ?err, "Failed to grant wallet access");
            Err(Status::internal("Failed to grant wallet access"))
        }
    }
}

async fn handle_revoke_wallet_access(
    actor: &ActorRef<UserAgentSession>,
    req: SdkClientRevokeWalletAccess,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    match actor
        .ask(HandleRevokeEvmWalletAccess { entries: req.accesses })
        .await
    {
        Ok(()) => {
            info!("Successfully revoked wallet access");
            Ok(None)
        }
        Err(err) => {
            warn!(error = ?err, "Failed to revoke wallet access");
            Err(Status::internal("Failed to revoke wallet access"))
        }
    }
}

async fn handle_list_wallet_access(
    actor: &ActorRef<UserAgentSession>,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    match actor.ask(HandleListWalletAccess {}).await {
        Ok(accesses) => Ok(Some(UserAgentResponsePayload::ListWalletAccessResponse(
            ListWalletAccessResponse {
                accesses: accesses.into_iter().map(|a| a.convert()).collect(),
            },
        ))),
        Err(err) => {
            warn!(error = ?err, "Failed to list wallet access");
            Err(Status::internal("Failed to list wallet access"))
        }
    }
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
