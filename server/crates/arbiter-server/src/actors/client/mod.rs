use arbiter_proto::{
    proto::client::{
        AuthChallenge, AuthChallengeRequest, AuthChallengeSolution, AuthOk, ClientRequest,
        ClientResponse,
        client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
    },
    transport::{Bi, DummyTransport},
};
use diesel::{ExpressionMethods as _, OptionalExtension as _, QueryDsl, dsl::update};
use diesel_async::RunQueryDsl;
use ed25519_dalek::VerifyingKey;
use kameo::Actor;
use tokio::select;
use tracing::{error, info};

use crate::{
    ServerContext,
    actors::client::state::{
        ChallengeContext, ClientEvents, ClientStateMachine, ClientStates, DummyContext,
    },
    db::{self, schema},
};

mod state;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ClientError {
    #[error("Expected message with payload")]
    MissingRequestPayload,
    #[error("Unexpected request payload")]
    UnexpectedRequestPayload,
    #[error("Invalid state for challenge solution")]
    InvalidStateForChallengeSolution,
    #[error("Expected pubkey to have specific length")]
    InvalidAuthPubkeyLength,
    #[error("Failed to convert pubkey to VerifyingKey")]
    InvalidAuthPubkeyEncoding,
    #[error("Invalid signature length")]
    InvalidSignatureLength,
    #[error("Public key not registered")]
    PublicKeyNotRegistered,
    #[error("Invalid challenge solution")]
    InvalidChallengeSolution,
    #[error("State machine error")]
    StateTransitionFailed,
    #[error("Database pool error")]
    DatabasePoolUnavailable,
    #[error("Database error")]
    DatabaseOperationFailed,
}

pub struct ClientActor<Transport>
where
    Transport: Bi<ClientRequest, Result<ClientResponse, ClientError>>,
{
    db: db::DatabasePool,
    state: ClientStateMachine<DummyContext>,
    transport: Transport,
}

impl<Transport> ClientActor<Transport>
where
    Transport: Bi<ClientRequest, Result<ClientResponse, ClientError>>,
{
    pub(crate) fn new(context: ServerContext, transport: Transport) -> Self {
        Self {
            db: context.db.clone(),
            state: ClientStateMachine::new(DummyContext),
            transport,
        }
    }

    fn transition(&mut self, event: ClientEvents) -> Result<(), ClientError> {
        self.state.process_event(event).map_err(|e| {
            error!(?e, "State transition failed");
            ClientError::StateTransitionFailed
        })?;
        Ok(())
    }

    pub async fn process_transport_inbound(&mut self, req: ClientRequest) -> Output {
        let msg = req.payload.ok_or_else(|| {
            error!(actor = "client", "Received message with no payload");
            ClientError::MissingRequestPayload
        })?;

        match msg {
            ClientRequestPayload::AuthChallengeRequest(req) => {
                self.handle_auth_challenge_request(req).await
            }
            ClientRequestPayload::AuthChallengeSolution(solution) => {
                self.handle_auth_challenge_solution(solution).await
            }
        }
    }

    async fn handle_auth_challenge_request(&mut self, req: AuthChallengeRequest) -> Output {
        let pubkey = req
            .pubkey
            .as_array()
            .ok_or(ClientError::InvalidAuthPubkeyLength)?;
        let pubkey = VerifyingKey::from_bytes(pubkey).map_err(|_err| {
            error!(?pubkey, "Failed to convert to VerifyingKey");
            ClientError::InvalidAuthPubkeyEncoding
        })?;

        self.transition(ClientEvents::AuthRequest)?;

        self.auth_with_challenge(pubkey, req.pubkey).await
    }

    async fn auth_with_challenge(&mut self, pubkey: VerifyingKey, pubkey_bytes: Vec<u8>) -> Output {
        let nonce: Option<i32> = {
            let mut db_conn = self.db.get().await.map_err(|e| {
                error!(error = ?e, "Database pool error");
                ClientError::DatabasePoolUnavailable
            })?;
            db_conn
                .exclusive_transaction(|conn| {
                    Box::pin(async move {
                        let current_nonce = schema::program_client::table
                            .filter(
                                schema::program_client::public_key.eq(pubkey.as_bytes().to_vec()),
                            )
                            .select(schema::program_client::nonce)
                            .first::<i32>(conn)
                            .await?;

                        update(schema::program_client::table)
                            .filter(
                                schema::program_client::public_key.eq(pubkey.as_bytes().to_vec()),
                            )
                            .set(schema::program_client::nonce.eq(current_nonce + 1))
                            .execute(conn)
                            .await?;

                        Result::<_, diesel::result::Error>::Ok(current_nonce)
                    })
                })
                .await
                .optional()
                .map_err(|e| {
                    error!(error = ?e, "Database error");
                    ClientError::DatabaseOperationFailed
                })?
        };

        let Some(nonce) = nonce else {
            error!(?pubkey, "Public key not found in database");
            return Err(ClientError::PublicKeyNotRegistered);
        };

        let challenge = AuthChallenge {
            pubkey: pubkey_bytes,
            nonce,
        };

        self.transition(ClientEvents::SentChallenge(ChallengeContext {
            challenge: challenge.clone(),
            key: pubkey,
        }))?;

        info!(
            ?pubkey,
            ?challenge,
            "Sent authentication challenge to client"
        );

        Ok(response(ClientResponsePayload::AuthChallenge(challenge)))
    }

    fn verify_challenge_solution(
        &self,
        solution: &AuthChallengeSolution,
    ) -> Result<(bool, &ChallengeContext), ClientError> {
        let ClientStates::WaitingForChallengeSolution(challenge_context) = self.state.state()
        else {
            error!("Received challenge solution in invalid state");
            return Err(ClientError::InvalidStateForChallengeSolution);
        };
        let formatted_challenge = arbiter_proto::format_challenge(
            challenge_context.challenge.nonce,
            &challenge_context.challenge.pubkey,
        );

        let signature = solution.signature.as_slice().try_into().map_err(|_| {
            error!(?solution, "Invalid signature length");
            ClientError::InvalidSignatureLength
        })?;

        let valid = challenge_context
            .key
            .verify_strict(&formatted_challenge, &signature)
            .is_ok();

        Ok((valid, challenge_context))
    }

    async fn handle_auth_challenge_solution(
        &mut self,
        solution: AuthChallengeSolution,
    ) -> Output {
        let (valid, challenge_context) = self.verify_challenge_solution(&solution)?;

        if valid {
            info!(
                ?challenge_context,
                "Client provided valid solution to authentication challenge"
            );
            self.transition(ClientEvents::ReceivedGoodSolution)?;
            Ok(response(ClientResponsePayload::AuthOk(AuthOk {})))
        } else {
            error!("Client provided invalid solution to authentication challenge");
            self.transition(ClientEvents::ReceivedBadSolution)?;
            Err(ClientError::InvalidChallengeSolution)
        }
    }
}

type Output = Result<ClientResponse, ClientError>;

fn response(payload: ClientResponsePayload) -> ClientResponse {
    ClientResponse {
        payload: Some(payload),
    }
}

impl<Transport> Actor for ClientActor<Transport>
where
    Transport: Bi<ClientRequest, Result<ClientResponse, ClientError>>,
{
    type Args = Self;

    type Error = ();

    async fn on_start(
        args: Self::Args,
        _: kameo::prelude::ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
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
                msg = self.transport.recv() => {
                    match msg {
                        Some(request) => {
                            match self.process_transport_inbound(request).await {
                                Ok(resp) => {
                                    if self.transport.send(Ok(resp)).await.is_err() {
                                        error!(actor = "client", reason = "channel closed", "send.failed");
                                        return Some(kameo::mailbox::Signal::Stop);
                                    }
                                }
                                Err(err) => {
                                    let _ = self.transport.send(Err(err)).await;
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

impl ClientActor<DummyTransport<ClientRequest, Result<ClientResponse, ClientError>>> {
    pub fn new_manual(db: db::DatabasePool) -> Self {
        Self {
            db,
            state: ClientStateMachine::new(DummyContext),
            transport: DummyTransport::new(),
        }
    }
}
