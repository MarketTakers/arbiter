use super::ClientConnection;
use crate::{
    actors::{
        GlobalActors,
        evm::{ClientSignTransaction, SignTransactionError},
        flow_coordinator::RegisterClient,
        vault::VaultState,
    },
    db,
    evm::VetError,
};

use alloy::{consensus::TxEip1559, primitives::Address, signers::Signature};
use kameo::{Actor, messages};
use tracing::error;

pub struct ClientSession {
    props: ClientConnection,
    client_id: i32,
}

impl ClientSession {
    pub(crate) fn new(props: ClientConnection, client_id: i32) -> Self {
        Self { props, client_id }
    }
}

#[messages]
impl ClientSession {
    #[message]
    pub(crate) async fn handle_query_vault_state(&mut self) -> Result<VaultState, Error> {
        use crate::actors::vault::GetState;

        let vault_state = match self.props.actors.vault.ask(GetState {}).await {
            Ok(state) => state,
            Err(err) => {
                error!(?err, actor = "client", "vault.query.failed");
                return Err(Error::Internal);
            }
        };

        Ok(vault_state)
    }

    #[message]
    pub(crate) async fn handle_sign_transaction(
        &mut self,
        wallet_address: Address,
        transaction: TxEip1559,
    ) -> Result<Signature, SignTransactionRpcError> {
        match self
            .props
            .actors
            .evm
            .ask(ClientSignTransaction {
                client_id: self.client_id,
                wallet_address,
                transaction,
            })
            .await
        {
            Ok(signature) => Ok(signature),
            Err(kameo::error::SendError::HandlerError(SignTransactionError::Vet(vet_error))) => {
                Err(SignTransactionRpcError::Vet(vet_error))
            }
            Err(err) => {
                error!(?err, "Failed to sign EVM transaction in client session");
                Err(SignTransactionRpcError::Internal)
            }
        }
    }
}

impl Actor for ClientSession {
    type Args = Self;

    type Error = Error;

    async fn on_start(
        args: Self::Args,
        this: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        args.props
            .actors
            .flow_coordinator
            .ask(RegisterClient { actor: this })
            .await
            .map_err(|_| Error::ConnectionRegistrationFailed)?;
        Ok(args)
    }
}

impl ClientSession {
    pub fn new_test(db: db::DatabasePool, actors: GlobalActors) -> Self {
        let props = ClientConnection::new(db, actors);
        Self {
            props,
            client_id: 0,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Connection registration failed")]
    ConnectionRegistrationFailed,
    #[error("Internal error")]
    Internal,
}

#[derive(Debug, thiserror::Error)]
pub enum SignTransactionRpcError {
    #[error("Policy evaluation failed")]
    Vet(#[from] VetError),

    #[error("Internal error")]
    Internal,
}
