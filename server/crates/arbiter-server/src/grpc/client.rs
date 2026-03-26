use arbiter_proto::{
    google::protobuf::Empty as ProtoEmpty,
    proto::client::{
        ClientRequest, ClientResponse, VaultState as ProtoVaultState,
        client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
    },
    transport::{Receiver, Sender, grpc::GrpcBi},
};
use kameo::{
    actor::{ActorRef, Spawn as _},
    error::SendError,
};
use tonic::Status;
use tracing::{info, warn};

use crate::{
    actors::{
        client::{
            self, ClientConnection,
            session::{
                ClientSession, Error, HandleQueryVaultState, HandleSignTransaction,
                SignTransactionRpcError,
            },
        },
        keyholder::KeyHolderState,
    },
    evm::{PolicyError, VetError, policies::EvalViolation},
    grpc::request_tracker::RequestTracker,
    utils::defer,
};

use alloy::{
    consensus::TxEip1559,
    primitives::{Address, U256},
    rlp::Decodable,
};
use arbiter_proto::proto::evm::{
    EvmError as ProtoEvmError, EvmSignTransactionResponse, EvalViolation as ProtoEvalViolation,
    GasLimitExceededViolation, NoMatchingGrantError, PolicyViolationsError,
    SpecificMeaning as ProtoSpecificMeaning, TokenInfo as ProtoTokenInfo,
    TransactionEvalError,
    evm_sign_transaction_response::Result as EvmSignTransactionResult,
    eval_violation::Kind as ProtoEvalViolationKind,
    specific_meaning::Meaning as ProtoSpecificMeaningKind,
    transaction_eval_error::Kind as ProtoTransactionEvalErrorKind,
};

mod auth;

fn u256_to_proto_bytes(value: U256) -> Vec<u8> {
    value.to_be_bytes::<32>().to_vec()
}

fn meaning_to_proto(meaning: crate::evm::policies::SpecificMeaning) -> ProtoSpecificMeaning {
    let kind = match meaning {
        crate::evm::policies::SpecificMeaning::EtherTransfer(meaning) => {
            ProtoSpecificMeaningKind::EtherTransfer(arbiter_proto::proto::evm::EtherTransferMeaning {
                to: meaning.to.to_vec(),
                value: u256_to_proto_bytes(meaning.value),
            })
        }
        crate::evm::policies::SpecificMeaning::TokenTransfer(meaning) => {
            ProtoSpecificMeaningKind::TokenTransfer(arbiter_proto::proto::evm::TokenTransferMeaning {
                token: Some(ProtoTokenInfo {
                    symbol: meaning.token.symbol.to_string(),
                    address: meaning.token.contract.to_vec(),
                    chain_id: meaning.token.chain,
                }),
                to: meaning.to.to_vec(),
                value: u256_to_proto_bytes(meaning.value),
            })
        }
    };

    ProtoSpecificMeaning {
        meaning: Some(kind),
    }
}

fn violation_to_proto(violation: EvalViolation) -> ProtoEvalViolation {
    let kind = match violation {
        EvalViolation::InvalidTarget { target } => ProtoEvalViolationKind::InvalidTarget(target.to_vec()),
        EvalViolation::GasLimitExceeded {
            max_gas_fee_per_gas,
            max_priority_fee_per_gas,
        } => ProtoEvalViolationKind::GasLimitExceeded(GasLimitExceededViolation {
            max_gas_fee_per_gas: max_gas_fee_per_gas.map(u256_to_proto_bytes),
            max_priority_fee_per_gas: max_priority_fee_per_gas.map(u256_to_proto_bytes),
        }),
        EvalViolation::RateLimitExceeded => ProtoEvalViolationKind::RateLimitExceeded(ProtoEmpty {}),
        EvalViolation::VolumetricLimitExceeded => {
            ProtoEvalViolationKind::VolumetricLimitExceeded(ProtoEmpty {})
        }
        EvalViolation::InvalidTime => ProtoEvalViolationKind::InvalidTime(ProtoEmpty {}),
        EvalViolation::InvalidTransactionType => {
            ProtoEvalViolationKind::InvalidTransactionType(ProtoEmpty {})
        }
    };

    ProtoEvalViolation { kind: Some(kind) }
}

fn eval_error_to_proto(err: VetError) -> Option<TransactionEvalError> {
    let kind = match err {
        VetError::ContractCreationNotSupported => {
            ProtoTransactionEvalErrorKind::ContractCreationNotSupported(ProtoEmpty {})
        }
        VetError::UnsupportedTransactionType => {
            ProtoTransactionEvalErrorKind::UnsupportedTransactionType(ProtoEmpty {})
        }
        VetError::Evaluated(meaning, policy_error) => match policy_error {
            PolicyError::NoMatchingGrant => {
                ProtoTransactionEvalErrorKind::NoMatchingGrant(NoMatchingGrantError {
                    meaning: Some(meaning_to_proto(meaning)),
                })
            }
            PolicyError::Violations(violations) => {
                ProtoTransactionEvalErrorKind::PolicyViolations(PolicyViolationsError {
                    meaning: Some(meaning_to_proto(meaning)),
                    violations: violations.into_iter().map(violation_to_proto).collect(),
                })
            }
            PolicyError::Pool(_) | PolicyError::Database(_) => {
                return None;
            }
        },
    };

    Some(TransactionEvalError { kind: Some(kind) })
}

fn decode_eip1559_transaction(payload: &[u8]) -> Result<TxEip1559, ()> {
    let mut body = payload;
    if let Some((prefix, rest)) = payload.split_first()
        && *prefix == 0x02
    {
        body = rest;
    }

    let mut cursor = body;
    let transaction = TxEip1559::decode(&mut cursor).map_err(|_| ())?;
    if !cursor.is_empty() {
        return Err(());
    }

    Ok(transaction)
}

async fn dispatch_loop(
    mut bi: GrpcBi<ClientRequest, ClientResponse>,
    actor: ActorRef<ClientSession>,
    mut request_tracker: RequestTracker,
) {
    loop {
        let Some(conn) = bi.recv().await else {
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

async fn dispatch_conn_message(
    bi: &mut GrpcBi<ClientRequest, ClientResponse>,
    actor: &ActorRef<ClientSession>,
    request_tracker: &mut RequestTracker,
    conn: Result<ClientRequest, Status>,
) -> Result<(), ()> {
    let conn = match conn {
        Ok(conn) => conn,
        Err(err) => {
            warn!(error = ?err, "Failed to receive client request");
            return Err(());
        }
    };

    let request_id = match request_tracker.request(conn.request_id) {
        Ok(request_id) => request_id,
        Err(err) => {
            let _ = bi.send(Err(err)).await;
            return Err(());
        }
    };
    let Some(payload) = conn.payload else {
        let _ = bi
            .send(Err(Status::invalid_argument(
                "Missing client request payload",
            )))
            .await;
        return Err(());
    };

    let payload = match payload {
        ClientRequestPayload::QueryVaultState(_) => ClientResponsePayload::VaultState(
            match actor.ask(HandleQueryVaultState {}).await {
                Ok(KeyHolderState::Unbootstrapped) => ProtoVaultState::Unbootstrapped,
                Ok(KeyHolderState::Sealed) => ProtoVaultState::Sealed,
                Ok(KeyHolderState::Unsealed) => ProtoVaultState::Unsealed,
                Err(SendError::HandlerError(Error::Internal)) => ProtoVaultState::Error,
                Err(err) => {
                    warn!(error = ?err, "Failed to query vault state");
                    ProtoVaultState::Error
                }
            }
            .into(),
        ),
        ClientRequestPayload::EvmSignTransaction(request) => {
            let wallet_address = match <[u8; 20]>::try_from(request.wallet_address.as_slice()) {
                Ok(address) => Address::from(address),
                Err(_) => {
                    let _ = bi
                        .send(Err(Status::invalid_argument("Invalid EVM wallet address")))
                        .await;
                    return Err(());
                }
            };

            let transaction = match decode_eip1559_transaction(&request.rlp_transaction) {
                Ok(transaction) => transaction,
                Err(()) => {
                    let _ = bi
                        .send(Err(Status::invalid_argument(
                            "Invalid EIP-1559 RLP transaction",
                        )))
                        .await;
                    return Err(());
                }
            };

            let response = match actor
                .ask(HandleSignTransaction {
                    wallet_address,
                    transaction,
                })
                .await
            {
                Ok(signature) => EvmSignTransactionResponse {
                    result: Some(EvmSignTransactionResult::Signature(signature.as_bytes().to_vec())),
                },
                Err(kameo::error::SendError::HandlerError(SignTransactionRpcError::Vet(vet_error))) => {
                    match eval_error_to_proto(vet_error) {
                        Some(eval_error) => EvmSignTransactionResponse {
                            result: Some(EvmSignTransactionResult::EvalError(eval_error)),
                        },
                        None => EvmSignTransactionResponse {
                            result: Some(EvmSignTransactionResult::Error(ProtoEvmError::Internal.into())),
                        },
                    }
                }
                Err(kameo::error::SendError::HandlerError(SignTransactionRpcError::Internal)) => {
                    EvmSignTransactionResponse {
                        result: Some(EvmSignTransactionResult::Error(ProtoEvmError::Internal.into())),
                    }
                }
                Err(err) => {
                    warn!(error = ?err, "Failed to sign EVM transaction");
                    EvmSignTransactionResponse {
                        result: Some(EvmSignTransactionResult::Error(ProtoEvmError::Internal.into())),
                    }
                }
            };

            ClientResponsePayload::EvmSignTransaction(response)
        }
        payload => {
            warn!(?payload, "Unsupported post-auth client request");
            let _ = bi
                .send(Err(Status::invalid_argument("Unsupported client request")))
                .await;
            return Err(());
        }
    };

    bi.send(Ok(ClientResponse {
        request_id: Some(request_id),
        payload: Some(payload),
    }))
    .await
    .map_err(|_| ())
}

pub async fn start(conn: ClientConnection, mut bi: GrpcBi<ClientRequest, ClientResponse>) {
    let mut conn = conn;
    let mut request_tracker = RequestTracker::default();
    let mut response_id = None;

    match auth::start(&mut conn, &mut bi, &mut request_tracker, &mut response_id).await {
        Ok(_) => {
            let actor =
                client::session::ClientSession::spawn(client::session::ClientSession::new(conn));
            let actor_for_cleanup = actor.clone();
            let _ = defer(move || {
                actor_for_cleanup.kill();
            });

            info!("Client authenticated successfully");
            dispatch_loop(bi, actor, request_tracker).await;
        }
        Err(e) => {
            let mut transport = auth::AuthTransportAdapter::new(
                &mut bi,
                &mut request_tracker,
                &mut response_id,
            );
            let _ = transport.send(Err(e.clone())).await;
            warn!(error = ?e, "Authentication failed");
        }
    }
}
