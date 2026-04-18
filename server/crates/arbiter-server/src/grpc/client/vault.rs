use crate::{
    actors::vault::VaultState,
    peers::client::session::{ClientSession, Error, HandleQueryVaultState},
};
use arbiter_proto::proto::{
    client::{
        client_response::Payload as ClientResponsePayload,
        vault::{
            self as proto_vault, request::Payload as VaultRequestPayload,
            response::Payload as VaultResponsePayload,
        },
    },
    shared::VaultState as ProtoVaultState,
};

use kameo::{actor::ActorRef, error::SendError};
use tonic::Status;
use tracing::warn;

pub(super) async fn dispatch(
    actor: &ActorRef<ClientSession>,
    req: proto_vault::Request,
) -> Result<ClientResponsePayload, Status> {
    let Some(payload) = req.payload else {
        return Err(Status::invalid_argument(
            "Missing client vault request payload",
        ));
    };

    match payload {
        VaultRequestPayload::QueryState(()) => {
            let state = match actor.ask(HandleQueryVaultState {}).await {
                Ok(VaultState::Unbootstrapped) => ProtoVaultState::Unbootstrapped,
                Ok(VaultState::Sealed) => ProtoVaultState::Sealed,
                Ok(VaultState::Unsealed) => ProtoVaultState::Unsealed,
                Err(SendError::HandlerError(Error::Internal)) => ProtoVaultState::Error,
                Err(err) => {
                    warn!(error = ?err, "Failed to query vault state");
                    ProtoVaultState::Error
                }
            };
            Ok(ClientResponsePayload::Vault(proto_vault::Response {
                payload: Some(VaultResponsePayload::State(state.into())),
            }))
        }
    }
}
