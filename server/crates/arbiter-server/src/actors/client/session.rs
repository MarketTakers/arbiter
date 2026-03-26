use kameo::{Actor, messages};
use tracing::error;

use alloy::{consensus::TxEip1559, primitives::Address, signers::Signature};

use crate::{
    actors::{
        GlobalActors,
        client::ClientConnection,
        evm::{ClientSignTransaction, SignTransactionError},
        keyholder::KeyHolderState,
        router::RegisterClient,
    },
    db,
    evm::VetError,
};

pub struct ClientSession {
    props: ClientConnection,
}

impl ClientSession {
    pub(crate) fn new(props: ClientConnection) -> Self {
        Self { props }
    }
}

#[messages]
impl ClientSession {
    #[message]
    pub(crate) async fn handle_query_vault_state(&mut self) -> Result<KeyHolderState, Error> {
        use crate::actors::keyholder::GetState;

        let vault_state = match self.props.actors.key_holder.ask(GetState {}).await {
            Ok(state) => state,
            Err(err) => {
                error!(?err, actor = "client", "keyholder.query.failed");
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
                client_id: self.props.client_id,
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
            .router
            .ask(RegisterClient { actor: this })
            .await
            .map_err(|_| Error::ConnectionRegistrationFailed)?;
        Ok(args)
    }
}

impl ClientSession {
    pub fn new_test(db: db::DatabasePool, actors: GlobalActors) -> Self {
        let props = ClientConnection::new(db, actors);
        Self { props }
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
