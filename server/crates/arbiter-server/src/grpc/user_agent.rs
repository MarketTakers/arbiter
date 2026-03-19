use tokio::sync::mpsc;

use arbiter_proto::{
    proto::{
        evm::{
            EtherTransferSettings as ProtoEtherTransferSettings, EvmError as ProtoEvmError,
            EvmGrantCreateRequest, EvmGrantCreateResponse, EvmGrantDeleteRequest,
            EvmGrantDeleteResponse, EvmGrantList, EvmGrantListResponse, GrantEntry,
            SharedSettings as ProtoSharedSettings, SpecificGrant as ProtoSpecificGrant,
            TokenTransferSettings as ProtoTokenTransferSettings,
            TransactionRateLimit as ProtoTransactionRateLimit,
            VolumeRateLimit as ProtoVolumeRateLimit, WalletCreateResponse, WalletEntry, WalletList,
            WalletListResponse, evm_grant_create_response::Result as EvmGrantCreateResult,
            evm_grant_delete_response::Result as EvmGrantDeleteResult,
            evm_grant_list_response::Result as EvmGrantListResult,
            specific_grant::Grant as ProtoSpecificGrantType,
            wallet_create_response::Result as WalletCreateResult,
            wallet_list_response::Result as WalletListResult,
        },
        user_agent::{
            BootstrapEncryptedKey as ProtoBootstrapEncryptedKey,
            BootstrapResult as ProtoBootstrapResult, ClientConnectionCancel,
            ClientConnectionRequest, UnsealEncryptedKey as ProtoUnsealEncryptedKey,
            UnsealResult as ProtoUnsealResult, UnsealStart, UserAgentRequest, UserAgentResponse,
            VaultState as ProtoVaultState, user_agent_request::Payload as UserAgentRequestPayload,
            user_agent_response::Payload as UserAgentResponsePayload,
        },
    },
    transport::{Error as TransportError, Receiver, Sender, grpc::GrpcBi},
};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use kameo::{
    actor::{ActorRef, Spawn as _},
    error::SendError,
};
use tonic::Status;
use tracing::{info, warn};

use crate::{
    actors::{
        keyholder::KeyHolderState,
        user_agent::{
            OutOfBand, UserAgentConnection, UserAgentSession,
            session::{
                BootstrapError, Error, HandleBootstrapEncryptedKey, HandleEvmWalletCreate,
                HandleEvmWalletList, HandleGrantCreate, HandleGrantDelete, HandleGrantList,
                HandleQueryVaultState, HandleUnsealEncryptedKey, HandleUnsealRequest, UnsealError,
            },
        },
    },
    evm::policies::{
        Grant, SharedGrantSettings, SpecificGrant, TransactionRateLimit, VolumeRateLimit,
        ether_transfer, token_transfers,
    },
    grpc::request_tracker::RequestTracker,
    utils::defer,
};
use alloy::primitives::{Address, U256};
mod auth;

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
                    return;
                };

                if send_out_of_band(&mut bi, oob).await.is_err() {
                    return;
                }
            }

            conn = bi.recv() => {
                let Some(conn) = conn else {
                    return;
                };

                if dispatch_conn_message(&mut bi, &actor, &mut request_tracker, conn)
                    .await
                    .is_err()
                {
                    return;
                }
            }
        }
    }
}

async fn dispatch_conn_message(
    bi: &mut GrpcBi<UserAgentRequest, UserAgentResponse>,
    actor: &ActorRef<UserAgentSession>,
    request_tracker: &mut RequestTracker,
    conn: Result<UserAgentRequest, Status>,
) -> Result<(), ()> {
    let conn = match conn {
        Ok(conn) => conn,
        Err(err) => {
            warn!(error = ?err, "Failed to receive user agent request");
            return Err(());
        }
    };

    let request_id = match request_tracker.request(conn.id) {
        Ok(request_id) => request_id,
        Err(err) => {
            let _ = bi.send(Err(err)).await;
            return Err(());
        }
    };

    let Some(payload) = conn.payload else {
        let _ = bi
            .send(Err(Status::invalid_argument(
                "Missing user-agent request payload",
            )))
            .await;
        return Err(());
    };

    let payload = match payload {
        UserAgentRequestPayload::UnsealStart(UnsealStart { client_pubkey }) => {
            let client_pubkey = match <[u8; 32]>::try_from(client_pubkey) {
                Ok(bytes) => x25519_dalek::PublicKey::from(bytes),
                Err(_) => {
                    let _ = bi
                        .send(Err(Status::invalid_argument("Invalid X25519 public key")))
                        .await;
                    return Err(());
                }
            };

            match actor.ask(HandleUnsealRequest { client_pubkey }).await {
                Ok(response) => UserAgentResponsePayload::UnsealStartResponse(
                    arbiter_proto::proto::user_agent::UnsealStartResponse {
                        server_pubkey: response.server_pubkey.as_bytes().to_vec(),
                    },
                ),
                Err(err) => {
                    warn!(error = ?err, "Failed to handle unseal start request");
                    let _ = bi
                        .send(Err(Status::internal("Failed to start unseal flow")))
                        .await;
                    return Err(());
                }
            }
        }
        UserAgentRequestPayload::UnsealEncryptedKey(ProtoUnsealEncryptedKey {
            nonce,
            ciphertext,
            associated_data,
        }) => UserAgentResponsePayload::UnsealResult(
            match actor
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
                    let _ = bi
                        .send(Err(Status::internal("Failed to unseal vault")))
                        .await;
                    return Err(());
                }
            }
            .into(),
        ),
        UserAgentRequestPayload::BootstrapEncryptedKey(ProtoBootstrapEncryptedKey {
            nonce,
            ciphertext,
            associated_data,
        }) => UserAgentResponsePayload::BootstrapResult(
            match actor
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
                    let _ = bi
                        .send(Err(Status::internal("Failed to bootstrap vault")))
                        .await;
                    return Err(());
                }
            }
            .into(),
        ),
        UserAgentRequestPayload::QueryVaultState(_) => UserAgentResponsePayload::VaultState(
            match actor.ask(HandleQueryVaultState {}).await {
                Ok(KeyHolderState::Unbootstrapped) => ProtoVaultState::Unbootstrapped,
                Ok(KeyHolderState::Sealed) => ProtoVaultState::Sealed,
                Ok(KeyHolderState::Unsealed) => ProtoVaultState::Unsealed,
                Err(err) => {
                    warn!(error = ?err, "Failed to query vault state");
                    ProtoVaultState::Error
                }
            }
            .into(),
        ),
        UserAgentRequestPayload::EvmWalletCreate(_) => UserAgentResponsePayload::EvmWalletCreate(
            EvmGrantOrWallet::wallet_create_response(actor.ask(HandleEvmWalletCreate {}).await),
        ),
        UserAgentRequestPayload::EvmWalletList(_) => UserAgentResponsePayload::EvmWalletList(
            EvmGrantOrWallet::wallet_list_response(actor.ask(HandleEvmWalletList {}).await),
        ),
        UserAgentRequestPayload::EvmGrantList(_) => UserAgentResponsePayload::EvmGrantList(
            EvmGrantOrWallet::grant_list_response(actor.ask(HandleGrantList {}).await),
        ),
        UserAgentRequestPayload::EvmGrantCreate(EvmGrantCreateRequest { shared, specific }) => {
            let (basic, grant) = match parse_grant_request(shared, specific) {
                Ok(values) => values,
                Err(status) => {
                    let _ = bi.send(Err(status)).await;
                    return Err(());
                }
            };

            UserAgentResponsePayload::EvmGrantCreate(EvmGrantOrWallet::grant_create_response(
                actor.ask(HandleGrantCreate { basic, grant }).await,
            ))
        }
        UserAgentRequestPayload::EvmGrantDelete(EvmGrantDeleteRequest { grant_id }) => {
            UserAgentResponsePayload::EvmGrantDelete(EvmGrantOrWallet::grant_delete_response(
                actor.ask(HandleGrantDelete { grant_id }).await,
            ))
        }
        payload => {
            warn!(?payload, "Unsupported post-auth user agent request");
            let _ = bi
                .send(Err(Status::invalid_argument(
                    "Unsupported user-agent request",
                )))
                .await;
            return Err(());
        }
    };

    bi.send(Ok(UserAgentResponse {
        id: Some(request_id),
        payload: Some(payload),
    }))
    .await
    .map_err(|_| ())
}

async fn send_out_of_band(
    bi: &mut GrpcBi<UserAgentRequest, UserAgentResponse>,
    oob: OutOfBand,
) -> Result<(), ()> {
    let payload = match oob {
        OutOfBand::ClientConnectionRequest { pubkey } => {
            UserAgentResponsePayload::ClientConnectionRequest(ClientConnectionRequest {
                pubkey: pubkey.to_bytes().to_vec(),
                info: None,
            })
        }
        OutOfBand::ClientConnectionCancel => {
            UserAgentResponsePayload::ClientConnectionCancel(ClientConnectionCancel {})
        }
    };

    bi.send(Ok(UserAgentResponse {
        id: None,
        payload: Some(payload),
    }))
    .await
    .map_err(|_| ())
}

fn parse_grant_request(
    shared: Option<ProtoSharedSettings>,
    specific: Option<ProtoSpecificGrant>,
) -> Result<(SharedGrantSettings, SpecificGrant), Status> {
    let shared = shared.ok_or_else(|| Status::invalid_argument("Missing shared grant settings"))?;
    let specific =
        specific.ok_or_else(|| Status::invalid_argument("Missing specific grant settings"))?;

    Ok((
        shared_settings_from_proto(shared)?,
        specific_grant_from_proto(specific)?,
    ))
}

fn shared_settings_from_proto(shared: ProtoSharedSettings) -> Result<SharedGrantSettings, Status> {
    Ok(SharedGrantSettings {
        visibility_id: shared.visibility_id,
        chain: shared.chain_id,
        valid_from: shared.valid_from.map(proto_timestamp_to_utc).transpose()?,
        valid_until: shared.valid_until.map(proto_timestamp_to_utc).transpose()?,
        max_gas_fee_per_gas: shared
            .max_gas_fee_per_gas
            .as_deref()
            .map(u256_from_proto_bytes)
            .transpose()?,
        max_priority_fee_per_gas: shared
            .max_priority_fee_per_gas
            .as_deref()
            .map(u256_from_proto_bytes)
            .transpose()?,
        rate_limit: shared.rate_limit.map(|limit| TransactionRateLimit {
            count: limit.count,
            window: chrono::Duration::seconds(limit.window_secs),
        }),
    })
}

fn specific_grant_from_proto(specific: ProtoSpecificGrant) -> Result<SpecificGrant, Status> {
    match specific.grant {
        Some(ProtoSpecificGrantType::EtherTransfer(ProtoEtherTransferSettings {
            targets,
            limit,
        })) => Ok(SpecificGrant::EtherTransfer(ether_transfer::Settings {
            target: targets
                .into_iter()
                .map(address_from_bytes)
                .collect::<Result<_, _>>()?,
            limit: volume_rate_limit_from_proto(limit.ok_or_else(|| {
                Status::invalid_argument("Missing ether transfer volume rate limit")
            })?)?,
        })),
        Some(ProtoSpecificGrantType::TokenTransfer(ProtoTokenTransferSettings {
            token_contract,
            target,
            volume_limits,
        })) => Ok(SpecificGrant::TokenTransfer(token_transfers::Settings {
            token_contract: address_from_bytes(token_contract)?,
            target: target.map(address_from_bytes).transpose()?,
            volume_limits: volume_limits
                .into_iter()
                .map(volume_rate_limit_from_proto)
                .collect::<Result<_, _>>()?,
        })),
        None => Err(Status::invalid_argument("Missing specific grant kind")),
    }
}

fn volume_rate_limit_from_proto(limit: ProtoVolumeRateLimit) -> Result<VolumeRateLimit, Status> {
    Ok(VolumeRateLimit {
        max_volume: u256_from_proto_bytes(&limit.max_volume)?,
        window: chrono::Duration::seconds(limit.window_secs),
    })
}

fn address_from_bytes(bytes: Vec<u8>) -> Result<Address, Status> {
    if bytes.len() != 20 {
        return Err(Status::invalid_argument("Invalid EVM address"));
    }

    Ok(Address::from_slice(&bytes))
}

fn u256_from_proto_bytes(bytes: &[u8]) -> Result<U256, Status> {
    if bytes.len() > 32 {
        return Err(Status::invalid_argument("Invalid U256 byte length"));
    }

    Ok(U256::from_be_slice(bytes))
}

fn proto_timestamp_to_utc(
    timestamp: prost_types::Timestamp,
) -> Result<chrono::DateTime<Utc>, Status> {
    Utc.timestamp_opt(timestamp.seconds, timestamp.nanos as u32)
        .single()
        .ok_or_else(|| Status::invalid_argument("Invalid timestamp"))
}

fn shared_settings_to_proto(shared: SharedGrantSettings) -> ProtoSharedSettings {
    ProtoSharedSettings {
        visibility_id: shared.visibility_id,
        chain_id: shared.chain,
        valid_from: shared.valid_from.map(|time| prost_types::Timestamp {
            seconds: time.timestamp(),
            nanos: time.timestamp_subsec_nanos() as i32,
        }),
        valid_until: shared.valid_until.map(|time| prost_types::Timestamp {
            seconds: time.timestamp(),
            nanos: time.timestamp_subsec_nanos() as i32,
        }),
        max_gas_fee_per_gas: shared
            .max_gas_fee_per_gas
            .map(|value| value.to_be_bytes::<32>().to_vec()),
        max_priority_fee_per_gas: shared
            .max_priority_fee_per_gas
            .map(|value| value.to_be_bytes::<32>().to_vec()),
        rate_limit: shared.rate_limit.map(|limit| ProtoTransactionRateLimit {
            count: limit.count,
            window_secs: limit.window.num_seconds(),
        }),
    }
}

fn specific_grant_to_proto(grant: SpecificGrant) -> ProtoSpecificGrant {
    let grant = match grant {
        SpecificGrant::EtherTransfer(settings) => {
            ProtoSpecificGrantType::EtherTransfer(ProtoEtherTransferSettings {
                targets: settings
                    .target
                    .into_iter()
                    .map(|address| address.to_vec())
                    .collect(),
                limit: Some(ProtoVolumeRateLimit {
                    max_volume: settings.limit.max_volume.to_be_bytes::<32>().to_vec(),
                    window_secs: settings.limit.window.num_seconds(),
                }),
            })
        }
        SpecificGrant::TokenTransfer(settings) => {
            ProtoSpecificGrantType::TokenTransfer(ProtoTokenTransferSettings {
                token_contract: settings.token_contract.to_vec(),
                target: settings.target.map(|address| address.to_vec()),
                volume_limits: settings
                    .volume_limits
                    .into_iter()
                    .map(|limit| ProtoVolumeRateLimit {
                        max_volume: limit.max_volume.to_be_bytes::<32>().to_vec(),
                        window_secs: limit.window.num_seconds(),
                    })
                    .collect(),
            })
        }
    };

    ProtoSpecificGrant { grant: Some(grant) }
}

struct EvmGrantOrWallet;

impl EvmGrantOrWallet {
    fn wallet_create_response<M>(
        result: Result<Address, SendError<M, Error>>,
    ) -> WalletCreateResponse {
        let result = match result {
            Ok(wallet) => WalletCreateResult::Wallet(WalletEntry {
                address: wallet.to_vec(),
            }),
            Err(err) => {
                warn!(error = ?err, "Failed to create EVM wallet");
                WalletCreateResult::Error(ProtoEvmError::Internal.into())
            }
        };

        WalletCreateResponse {
            result: Some(result),
        }
    }

    fn wallet_list_response<M>(
        result: Result<Vec<Address>, SendError<M, Error>>,
    ) -> WalletListResponse {
        let result = match result {
            Ok(wallets) => WalletListResult::Wallets(WalletList {
                wallets: wallets
                    .into_iter()
                    .map(|wallet| WalletEntry {
                        address: wallet.to_vec(),
                    })
                    .collect(),
            }),
            Err(err) => {
                warn!(error = ?err, "Failed to list EVM wallets");
                WalletListResult::Error(ProtoEvmError::Internal.into())
            }
        };

        WalletListResponse {
            result: Some(result),
        }
    }

    fn grant_create_response<M>(
        result: Result<i32, SendError<M, Error>>,
    ) -> EvmGrantCreateResponse {
        let result = match result {
            Ok(grant_id) => EvmGrantCreateResult::GrantId(grant_id),
            Err(err) => {
                warn!(error = ?err, "Failed to create EVM grant");
                EvmGrantCreateResult::Error(ProtoEvmError::Internal.into())
            }
        };

        EvmGrantCreateResponse {
            result: Some(result),
        }
    }

    fn grant_delete_response<M>(result: Result<(), SendError<M, Error>>) -> EvmGrantDeleteResponse {
        let result = match result {
            Ok(()) => EvmGrantDeleteResult::Ok(()),
            Err(err) => {
                warn!(error = ?err, "Failed to delete EVM grant");
                EvmGrantDeleteResult::Error(ProtoEvmError::Internal.into())
            }
        };

        EvmGrantDeleteResponse {
            result: Some(result),
        }
    }

    fn grant_list_response<M>(
        result: Result<Vec<Grant<SpecificGrant>>, SendError<M, Error>>,
    ) -> EvmGrantListResponse {
        let result = match result {
            Ok(grants) => EvmGrantListResult::Grants(EvmGrantList {
                grants: grants
                    .into_iter()
                    .map(|grant| GrantEntry {
                        id: grant.id,
                        visibility_id: grant.shared.visibility_id,
                        shared: Some(shared_settings_to_proto(grant.shared)),
                        specific: Some(specific_grant_to_proto(grant.settings)),
                    })
                    .collect(),
            }),
            Err(err) => {
                warn!(error = ?err, "Failed to list EVM grants");
                EvmGrantListResult::Error(ProtoEvmError::Internal.into())
            }
        };

        EvmGrantListResponse {
            result: Some(result),
        }
    }
}

pub async fn start(
    mut conn: UserAgentConnection,
    mut bi: GrpcBi<UserAgentRequest, UserAgentResponse>,
) {
    let mut request_tracker = RequestTracker::default();

    let pubkey = match auth::start(&mut conn, &mut bi, &mut request_tracker).await
    {
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

    let _ = defer(move || {
        actor_for_cleanup.kill();
    });

    info!(?pubkey, "User authenticated successfully");
    dispatch_loop(bi, actor, oob_receiver, request_tracker).await;
}
