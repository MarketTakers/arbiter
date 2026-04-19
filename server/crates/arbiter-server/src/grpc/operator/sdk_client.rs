use crate::{
    db::models::NewEvmWalletAccess,
    grpc::Convert,
    peers::operator::{
        OutOfBand, OperatorSession,
        session::handlers::{
            HandleGrantEvmWalletAccess, HandleListWalletAccess, HandleNewClientApprove,
            HandleRevokeEvmWalletAccess, HandleSdkClientList,
        },
    },
};
use arbiter_crypto::authn;
use arbiter_proto::proto::{
    shared::ClientInfo as ProtoClientMetadata,
    operator::{
        sdk_client::{
            self as proto_sdk_client, ConnectionCancel as ProtoSdkClientConnectionCancel,
            ConnectionRequest as ProtoSdkClientConnectionRequest,
            ConnectionResponse as ProtoSdkClientConnectionResponse, Entry as ProtoSdkClientEntry,
            Error as ProtoSdkClientError, GrantWalletAccess as ProtoSdkClientGrantWalletAccess,
            List as ProtoSdkClientList, ListResponse as ProtoSdkClientListResponse,
            ListWalletAccessResponse, RevokeWalletAccess as ProtoSdkClientRevokeWalletAccess,
            list_response::Result as ProtoSdkClientListResult,
            request::Payload as SdkClientRequestPayload,
            response::Payload as SdkClientResponsePayload,
        },
        operator_response::Payload as OperatorResponsePayload,
    },
};

use kameo::actor::ActorRef;
use tonic::Status;
use tracing::{info, warn};

const fn wrap_sdk_client_response(payload: SdkClientResponsePayload) -> OperatorResponsePayload {
    OperatorResponsePayload::SdkClient(proto_sdk_client::Response {
        payload: Some(payload),
    })
}

pub(super) fn out_of_band_payload(oob: OutOfBand) -> OperatorResponsePayload {
    match oob {
        OutOfBand::ClientConnectionRequest { profile } => wrap_sdk_client_response(
            SdkClientResponsePayload::ConnectionRequest(ProtoSdkClientConnectionRequest {
                pubkey: profile.pubkey.to_bytes(),
                info: Some(ProtoClientMetadata {
                    name: profile.metadata.name,
                    description: profile.metadata.description,
                    version: profile.metadata.version,
                }),
            }),
        ),
        OutOfBand::ClientConnectionCancel { pubkey } => wrap_sdk_client_response(
            SdkClientResponsePayload::ConnectionCancel(ProtoSdkClientConnectionCancel {
                pubkey: pubkey.to_bytes(),
            }),
        ),
    }
}

pub(super) async fn dispatch(
    actor: &ActorRef<OperatorSession>,
    req: proto_sdk_client::Request,
) -> Result<Option<OperatorResponsePayload>, Status> {
    let Some(payload) = req.payload else {
        return Err(Status::invalid_argument(
            "Missing SDK client request payload",
        ));
    };

    match payload {
        SdkClientRequestPayload::ConnectionResponse(resp) => {
            handle_connection_response(actor, resp).await
        }
        SdkClientRequestPayload::Revoke(_) => Err(Status::unimplemented(
            "SdkClientRevoke is not yet implemented",
        )),
        SdkClientRequestPayload::List(()) => handle_list(actor).await,
        SdkClientRequestPayload::GrantWalletAccess(req) => {
            handle_grant_wallet_access(actor, req).await
        }
        SdkClientRequestPayload::RevokeWalletAccess(req) => {
            handle_revoke_wallet_access(actor, req).await
        }
        SdkClientRequestPayload::ListWalletAccess(()) => handle_list_wallet_access(actor).await,
    }
}

async fn handle_connection_response(
    actor: &ActorRef<OperatorSession>,
    resp: ProtoSdkClientConnectionResponse,
) -> Result<Option<OperatorResponsePayload>, Status> {
    let pubkey = authn::PublicKey::try_from(resp.pubkey.as_slice())
        .map_err(|()| Status::invalid_argument("Invalid ML-DSA public key"))?;

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

async fn handle_list(
    actor: &ActorRef<OperatorSession>,
) -> Result<Option<OperatorResponsePayload>, Status> {
    let result = match actor.ask(HandleSdkClientList {}).await {
        Ok(clients) => ProtoSdkClientListResult::Clients(ProtoSdkClientList {
            clients: clients
                .into_iter()
                .map(|(client, metadata)| ProtoSdkClientEntry {
                    id: client.id,
                    pubkey: client.public_key.clone(),
                    info: Some(ProtoClientMetadata {
                        name: metadata.name,
                        description: metadata.description,
                        version: metadata.version,
                    }),
                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::as_conversions,
                        reason = "fixme! #84"
                    )]
                    created_at: client.created_at.0.timestamp() as i32,
                })
                .collect(),
        }),
        Err(err) => {
            warn!(error = ?err, "Failed to list SDK clients");
            ProtoSdkClientListResult::Error(ProtoSdkClientError::Internal.into())
        }
    };
    Ok(Some(wrap_sdk_client_response(
        SdkClientResponsePayload::List(ProtoSdkClientListResponse {
            result: Some(result),
        }),
    )))
}

async fn handle_grant_wallet_access(
    actor: &ActorRef<OperatorSession>,
    req: ProtoSdkClientGrantWalletAccess,
) -> Result<Option<OperatorResponsePayload>, Status> {
    let entries: Vec<NewEvmWalletAccess> = req.accesses.into_iter().map(Convert::convert).collect();
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
    actor: &ActorRef<OperatorSession>,
    req: ProtoSdkClientRevokeWalletAccess,
) -> Result<Option<OperatorResponsePayload>, Status> {
    match actor
        .ask(HandleRevokeEvmWalletAccess {
            entries: req.accesses,
        })
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
    actor: &ActorRef<OperatorSession>,
) -> Result<Option<OperatorResponsePayload>, Status> {
    match actor.ask(HandleListWalletAccess {}).await {
        Ok(accesses) => Ok(Some(wrap_sdk_client_response(
            SdkClientResponsePayload::ListWalletAccess(ListWalletAccessResponse {
                accesses: accesses.into_iter().map(Convert::convert).collect(),
            }),
        ))),
        Err(err) => {
            warn!(error = ?err, "Failed to list wallet access");
            Err(Status::internal("Failed to list wallet access"))
        }
    }
}
