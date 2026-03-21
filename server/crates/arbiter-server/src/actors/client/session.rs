use kameo::{Actor, messages};
use tracing::error;

use crate::{
    actors::{
        GlobalActors, client::ClientConnection, flow_coordinator::RegisterClient,
        keyholder::KeyHolderState,
    },
    db,
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
