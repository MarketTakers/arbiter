use arbiter_proto::proto::{
    evm::{
        EvmError as ProtoEvmError, EvmGrantCreateRequest, EvmGrantCreateResponse,
        EvmGrantDeleteRequest, EvmGrantDeleteResponse, EvmGrantList, EvmGrantListResponse,
        EvmSignTransactionResponse, GrantEntry, WalletCreateResponse, WalletEntry, WalletList,
        WalletListResponse, evm_grant_create_response::Result as EvmGrantCreateResult,
        evm_grant_delete_response::Result as EvmGrantDeleteResult,
        evm_grant_list_response::Result as EvmGrantListResult,
        evm_sign_transaction_response::Result as EvmSignTransactionResult,
        wallet_create_response::Result as WalletCreateResult,
        wallet_list_response::Result as WalletListResult,
    },
    user_agent::{
        evm::{
            self as proto_evm, SignTransactionRequest as ProtoSignTransactionRequest,
            request::Payload as EvmRequestPayload, response::Payload as EvmResponsePayload,
        },
        user_agent_response::Payload as UserAgentResponsePayload,
    },
};
use kameo::actor::ActorRef;
use tonic::Status;
use tracing::warn;

use crate::{
    actors::user_agent::{
        UserAgentSession,
        session::connection::{
            GrantMutationError, HandleEvmWalletCreate, HandleEvmWalletList, HandleGrantCreate,
            HandleGrantDelete, HandleGrantList, HandleSignTransaction,
            SignTransactionError as SessionSignTransactionError,
        },
    },
    grpc::{
        Convert, TryConvert,
        common::inbound::{RawEvmAddress, RawEvmTransaction},
    },
};

fn wrap_evm_response(payload: EvmResponsePayload) -> UserAgentResponsePayload {
    UserAgentResponsePayload::Evm(proto_evm::Response {
        payload: Some(payload),
    })
}

pub(super) async fn dispatch(
    actor: &ActorRef<UserAgentSession>,
    req: proto_evm::Request,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let Some(payload) = req.payload else {
        return Err(Status::invalid_argument("Missing EVM request payload"));
    };

    match payload {
        EvmRequestPayload::WalletCreate(_) => handle_wallet_create(actor).await,
        EvmRequestPayload::WalletList(_) => handle_wallet_list(actor).await,
        EvmRequestPayload::GrantCreate(req) => handle_grant_create(actor, req).await,
        EvmRequestPayload::GrantDelete(req) => handle_grant_delete(actor, req).await,
        EvmRequestPayload::GrantList(_) => handle_grant_list(actor).await,
        EvmRequestPayload::SignTransaction(req) => handle_sign_transaction(actor, req).await,
    }
}

async fn handle_wallet_create(
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
    Ok(Some(wrap_evm_response(EvmResponsePayload::WalletCreate(
        WalletCreateResponse {
            result: Some(result),
        },
    ))))
}

async fn handle_wallet_list(
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
    Ok(Some(wrap_evm_response(EvmResponsePayload::WalletList(
        WalletListResponse {
            result: Some(result),
        },
    ))))
}

async fn handle_grant_list(
    actor: &ActorRef<UserAgentSession>,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let result = match actor.ask(HandleGrantList {}).await {
        Ok(grants) => EvmGrantListResult::Grants(EvmGrantList {
            grants: grants
                .into_iter()
                .map(|grant| GrantEntry {
                    id: grant.common_settings_id,
                    wallet_access_id: grant.settings.shared.wallet_access_id,
                    shared: Some(grant.settings.shared.convert()),
                    specific: Some(grant.settings.specific.convert()),
                })
                .collect(),
        }),
        Err(err) => {
            warn!(error = ?err, "Failed to list EVM grants");
            EvmGrantListResult::Error(ProtoEvmError::Internal.into())
        }
    };
    Ok(Some(wrap_evm_response(EvmResponsePayload::GrantList(
        EvmGrantListResponse {
            result: Some(result),
        },
    ))))
}

async fn handle_grant_create(
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
        Err(kameo::error::SendError::HandlerError(GrantMutationError::VaultSealed)) => {
            EvmGrantCreateResult::Error(ProtoEvmError::VaultSealed.into())
        }
        Err(err) => {
            warn!(error = ?err, "Failed to create EVM grant");
            EvmGrantCreateResult::Error(ProtoEvmError::Internal.into())
        }
    };
    Ok(Some(wrap_evm_response(EvmResponsePayload::GrantCreate(
        EvmGrantCreateResponse {
            result: Some(result),
        },
    ))))
}

async fn handle_grant_delete(
    actor: &ActorRef<UserAgentSession>,
    req: EvmGrantDeleteRequest,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let result = match actor
        .ask(HandleGrantDelete {
            grant_id: req.grant_id,
            wallet_access_id: req.wallet_access_id,
        })
        .await
    {
        Ok(()) => EvmGrantDeleteResult::Ok(()),
        Err(kameo::error::SendError::HandlerError(GrantMutationError::VaultSealed)) => {
            EvmGrantDeleteResult::Error(ProtoEvmError::VaultSealed.into())
        }
        Err(err) => {
            warn!(error = ?err, "Failed to delete EVM grant");
            EvmGrantDeleteResult::Error(ProtoEvmError::Internal.into())
        }
    };
    Ok(Some(wrap_evm_response(EvmResponsePayload::GrantDelete(
        EvmGrantDeleteResponse {
            result: Some(result),
        },
    ))))
}

async fn handle_sign_transaction(
    actor: &ActorRef<UserAgentSession>,
    req: ProtoSignTransactionRequest,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let request = req
        .request
        .ok_or_else(|| Status::invalid_argument("Missing sign transaction request"))?;
    let wallet_address = RawEvmAddress(request.wallet_address).try_convert()?;
    let transaction = RawEvmTransaction(request.rlp_transaction).try_convert()?;

    let response = match actor
        .ask(HandleSignTransaction {
            client_id: req.client_id,
            wallet_address,
            transaction,
        })
        .await
    {
        Ok(signature) => EvmSignTransactionResponse {
            result: Some(EvmSignTransactionResult::Signature(
                signature.as_bytes().to_vec(),
            )),
        },
        Err(kameo::error::SendError::HandlerError(SessionSignTransactionError::Vet(vet_error))) => {
            EvmSignTransactionResponse {
                result: Some(vet_error.convert()),
            }
        }
        Err(kameo::error::SendError::HandlerError(SessionSignTransactionError::Internal)) => {
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

    Ok(Some(wrap_evm_response(
        EvmResponsePayload::SignTransaction(response),
    )))
}
