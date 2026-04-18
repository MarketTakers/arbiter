use arbiter_proto::proto::shared::VaultState as ProtoVaultState;
use arbiter_proto::proto::user_agent::{
    user_agent_response::Payload as UserAgentResponsePayload,
    vault::{self as proto_vault, request::Payload as VaultRequestPayload, response::Payload as VaultResponsePayload},
};
use kameo::actor::ActorRef;
use tonic::Status;
use tracing::warn;

use crate::{
    actors::vault::VaultState,
    peers::user_agent::{UserAgentSession, session::handlers::HandleQueryVaultState},
};

fn wrap_vault_response(payload: VaultResponsePayload) -> UserAgentResponsePayload {
    UserAgentResponsePayload::Vault(proto_vault::Response {
        payload: Some(payload),
    })
}

pub(super) async fn dispatch(
    actor: &ActorRef<UserAgentSession>,
    req: proto_vault::Request,
) -> Result<Option<UserAgentResponsePayload>, Status> {
    let Some(payload) = req.payload else {
        return Err(Status::invalid_argument("Missing vault request payload"));
    };

    match payload {
        VaultRequestPayload::QueryState(_) => handle_query_vault_state(actor).await,
        VaultRequestPayload::Unseal(_) | VaultRequestPayload::Bootstrap(_) => {
            Err(Status::permission_denied(
                "Vault is already unsealed; unseal/bootstrap not permitted in session",
            ))
        }
    }
}

async fn handle_query_vault_state(
    actor: &ActorRef<UserAgentSession>,
) -> Result<Option<UserAgentResponsePayload>, Status> {
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
