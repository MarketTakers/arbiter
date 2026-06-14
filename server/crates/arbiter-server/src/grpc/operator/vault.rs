use crate::{
    actors::vault::VaultState,
    peers::operator::{
        OperatorSession,
        session::handlers::{
            HandleContributeRecoveryRekeyPassphrase, HandleContributeRekeyPassphrase,
            HandleQueryVaultState,
        },
    },
};
use arbiter_proto::{
    proto::operator::{
        operator_response::Payload as OperatorResponsePayload,
        vault::{
            self as proto_vault,
            rekey::{self as proto_rekey, RekeyResult as ProtoRekeyResult},
            request::Payload as VaultRequestPayload,
            response::Payload as VaultResponsePayload,
        },
    },
    proto::shared::VaultState as ProtoVaultState,
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
        VaultRequestPayload::Rekey(req) => handle_rekey(actor, req).await,
        VaultRequestPayload::Unseal(_) | VaultRequestPayload::Bootstrap(_) => {
            Err(Status::permission_denied(
                "Vault is already unsealed; unseal/bootstrap not permitted in session",
            ))
        }
    }
}

async fn handle_rekey(
    actor: &ActorRef<OperatorSession>,
    req: proto_rekey::Request,
) -> Result<Option<OperatorResponsePayload>, Status> {
    use arbiter_proto::proto::operator::vault::rekey::request::Payload as RekeyPayload;

    let payload = req
        .payload
        .ok_or_else(|| Status::invalid_argument("Missing rekey payload"))?;

    let done: bool = match payload {
        RekeyPayload::ContributePassphrase(cp) => actor
            .ask(HandleContributeRekeyPassphrase {
                passphrase: cp.passphrase,
            })
            .await
            .map_err(|e| {
                warn!(?e, "rekey passphrase contribution failed");
                Status::internal("Rekey contribution failed")
            })?,
        RekeyPayload::ContributeRecoveryPassphrase(crp) => actor
            .ask(HandleContributeRecoveryRekeyPassphrase {
                recovery_operator_id: crp.recovery_operator_id,
                passphrase: crp.passphrase,
            })
            .await
            .map_err(|e| {
                warn!(?e, "rekey recovery passphrase contribution failed");
                Status::internal("Rekey recovery contribution failed")
            })?,
    };

    let proto_result = if done {
        ProtoRekeyResult::Success
    } else {
        ProtoRekeyResult::AwaitingContributions
    };

    Ok(Some(wrap_vault_response(VaultResponsePayload::Rekey(
        proto_rekey::Response {
            result: proto_result.into(),
        },
    ))))
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
