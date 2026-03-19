use alloy::{consensus::TxEip1559, primitives::Address, rlp::Decodable};
use arbiter_proto::proto::{
    client::{
        ClientRequest, ClientResponse, client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
    },
    evm::{
        EvmError, EvmSignTransactionResponse, evm_sign_transaction_response::Result as SignResult,
    },
};
use kameo::Actor;
use tokio::select;
use tracing::{error, info};

use crate::{
    actors::{
        GlobalActors,
        client::{ClientConnection, ClientError, auth::ClientId},
        evm::ClientSignTransaction,
        router::RegisterClient,
    },
    db,
};

pub struct ClientSession {
    props: ClientConnection,
    client_id: ClientId,
}

impl ClientSession {
    pub(crate) fn new(props: ClientConnection, client_id: ClientId) -> Self {
        Self { props, client_id }
    }

    pub async fn process_transport_inbound(&mut self, req: ClientRequest) -> Output {
        let msg = req.payload.ok_or_else(|| {
            error!(actor = "client", "Received message with no payload");
            ClientError::MissingRequestPayload
        })?;

        match msg {
            ClientRequestPayload::EvmSignTransaction(sign_req) => {
                let wallet_address: [u8; 20] = sign_req
                    .wallet_address
                    .try_into()
                    .map_err(|_| ClientError::UnexpectedRequestPayload)?;

                let mut rlp_bytes: &[u8] = &sign_req.rlp_transaction;
                let tx = TxEip1559::decode(&mut rlp_bytes)
                    .map_err(|_| ClientError::UnexpectedRequestPayload)?;

                let result = self
                    .props
                    .actors
                    .evm
                    .ask(ClientSignTransaction {
                        client_id: self.client_id.as_i32(),
                        wallet_address: Address::from_slice(&wallet_address),
                        transaction: tx,
                    })
                    .await;

                let response_result = match result {
                    Ok(signature) => SignResult::Signature(signature.as_bytes().to_vec()),
                    Err(err) => {
                        error!(?err, "client sign transaction failed");
                        SignResult::Error(EvmError::Internal.into())
                    }
                };

                Ok(ClientResponse {
                    payload: Some(ClientResponsePayload::EvmSignTransaction(
                        EvmSignTransactionResponse {
                            result: Some(response_result),
                        },
                    )),
                })
            }
            _ => Err(ClientError::UnexpectedRequestPayload),
        }
    }
}

type Output = Result<ClientResponse, ClientError>;

impl Actor for ClientSession {
    type Args = Self;

    type Error = ClientError;

    async fn on_start(
        args: Self::Args,
        this: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        args.props
            .actors
            .router
            .ask(RegisterClient { actor: this })
            .await
            .map_err(|_| ClientError::ConnectionRegistrationFailed)?;
        Ok(args)
    }

    async fn next(
        &mut self,
        _actor_ref: kameo::prelude::WeakActorRef<Self>,
        mailbox_rx: &mut kameo::prelude::MailboxReceiver<Self>,
    ) -> Option<kameo::mailbox::Signal<Self>> {
        loop {
            select! {
                signal = mailbox_rx.recv() => {
                    return signal;
                }
                msg = self.props.transport.recv() => {
                    match msg {
                        Some(request) => {
                            match self.process_transport_inbound(request).await {
                                Ok(resp) => {
                                    if self.props.transport.send(Ok(resp)).await.is_err() {
                                        error!(actor = "client", reason = "channel closed", "send.failed");
                                        return Some(kameo::mailbox::Signal::Stop);
                                    }
                                }
                                Err(err) => {
                                    let _ = self.props.transport.send(Err(err)).await;
                                    return Some(kameo::mailbox::Signal::Stop);
                                }
                            }
                        }
                        None => {
                            info!(actor = "client", "transport.closed");
                            return Some(kameo::mailbox::Signal::Stop);
                        }
                    }
                }
            }
        }
    }
}

impl ClientSession {
    pub fn new_test(db: db::DatabasePool, actors: GlobalActors) -> Self {
        use arbiter_proto::transport::DummyTransport;
        let transport: super::Transport = Box::new(DummyTransport::new());
        let props = ClientConnection::new(db, transport, actors);
        Self {
            props,
            client_id: ClientId::new(0),
        }
    }
}
