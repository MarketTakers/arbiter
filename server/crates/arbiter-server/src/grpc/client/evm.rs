use arbiter_proto::proto::{
    client::{
        client_response::Payload as ClientResponsePayload,
        evm::{
            self as proto_evm, request::Payload as EvmRequestPayload,
            response::Payload as EvmResponsePayload,
        },
    },
    evm::{
        EvmError as ProtoEvmError, EvmSignTransactionResponse,
        evm_sign_transaction_response::Result as EvmSignTransactionResult,
    },
};
use kameo::actor::ActorRef;
use tonic::Status;
use tracing::warn;

use crate::{
    actors::client::session::{ClientSession, HandleSignTransaction, SignTransactionRpcError},
    grpc::{
        Convert, TryConvert,
        common::inbound::{RawEvmAddress, RawEvmTransaction},
    },
};

const fn wrap_response(payload: EvmResponsePayload) -> ClientResponsePayload {
    ClientResponsePayload::Evm(proto_evm::Response {
        payload: Some(payload),
    })
}

pub(super) async fn dispatch(
    actor: &ActorRef<ClientSession>,
    req: proto_evm::Request,
) -> Result<ClientResponsePayload, Status> {
    let Some(payload) = req.payload else {
        return Err(Status::invalid_argument(
            "Missing client EVM request payload",
        ));
    };

    match payload {
        EvmRequestPayload::SignTransaction(request) => {
            let address = RawEvmAddress(request.wallet_address).try_convert()?;
            let transaction = RawEvmTransaction(request.rlp_transaction).try_convert()?;

            let response = match actor
                .ask(HandleSignTransaction {
                    wallet_address: address,
                    transaction,
                })
                .await
            {
                Ok(signature) => EvmSignTransactionResponse {
                    result: Some(EvmSignTransactionResult::Signature(
                        signature.as_bytes().to_vec(),
                    )),
                },
                Err(kameo::error::SendError::HandlerError(SignTransactionRpcError::Vet(
                    vet_error,
                ))) => EvmSignTransactionResponse {
                    result: Some(vet_error.convert()),
                },
                Err(kameo::error::SendError::HandlerError(SignTransactionRpcError::Internal)) => {
                    EvmSignTransactionResponse {
                        result: Some(EvmSignTransactionResult::Error(
                            ProtoEvmError::Internal.into(),
                        )),
                    }
                }
                Err(err) => {
                    warn!(error = ?err, "Failed to sign EVM transaction");
                    EvmSignTransactionResponse {
                        result: Some(EvmSignTransactionResult::Error(
                            ProtoEvmError::Internal.into(),
                        )),
                    }
                }
            };

            Ok(wrap_response(EvmResponsePayload::SignTransaction(response)))
        }
        EvmRequestPayload::AnalyzeTransaction(_) => Err(Status::unimplemented(
            "EVM transaction analysis is not yet implemented",
        )),
    }
}
