use crate::{
    actors::vault::VaultState,
    peers::operator::{OperatorSession, session::handlers::HandleQueryVaultState},
};
use arbiter_proto::{
    proto::shared::VaultState as ProtoVaultState,
    proto::operator::{
        operator_response::Payload as OperatorResponsePayload,
        vault::{
            self as proto_vault, request::Payload as VaultRequestPayload,
            response::Payload as VaultResponsePayload,
        },
    },
};

use kameo::actor::ActorRef;
use tonic::Status;
use tracing::warn;

const fn wrap_vault_response(payload: VaultResponsePayload) -> OperatorResponsePayload {
    OperatorResponsePayload::Vault(proto_vault::Response {
        payload: Some(payload),
    })
}

pub(super) async fn dispatch(
    actor: &ActorRef<OperatorSession>,
    req: proto_vault::Request,
) -> Result<Option<OperatorResponsePayload>, Status> {
    let Some(payload) = req.payload else {
        return Err(Status::invalid_argument("Missing vault request payload"));
    };

    match payload {
        VaultRequestPayload::QueryState(()) => handle_query_vault_state(actor).await,
        VaultRequestPayload::Unseal(_) | VaultRequestPayload::Bootstrap(_) => {
            Err(Status::permission_denied(
                "Vault is already unsealed; unseal/bootstrap not permitted in session",
            ))
        }
    }
}

async fn handle_query_vault_state(
    actor: &ActorRef<OperatorSession>,
) -> Result<Option<OperatorResponsePayload>, Status> {
    let state = match actor.ask(HandleQueryVaultState {}).await {
        Ok(VaultState::Unbootstrapped) => ProtoVaultState::Unbootstrapped,
        Ok(VaultState::Sealed) => ProtoVaultState::Sealed,
        Ok(VaultState::Unsealed) => ProtoVaultState::Unsealed,
        Err(err) => {
            warn!(error = ?err, "Failed to query vault state");
            ProtoVaultState::Error
        }
    };
    Ok(Some(wrap_vault_response(VaultResponsePayload::State(
        state.into(),
    ))))
}
