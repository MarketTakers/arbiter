use arbiter_proto::{
    format_challenge,
    proto::user_agent::{
        AuthChallengeRequest, AuthChallengeSolution, AuthOk, KeyType as ProtoKeyType,
        UserAgentRequest, UserAgentResponse,
        user_agent_request::Payload as UserAgentRequestPayload,
        user_agent_response::Payload as UserAgentResponsePayload,
    },
    transport::Bi,
};
use kameo::{Actor, actor::ActorRef};
use smlang::statemachine;
use tokio::select;
use tracing::{error, info};

/// Signing key variants supported by the user-agent auth protocol.
pub enum SigningKeyEnum {
    Ed25519(ed25519_dalek::SigningKey),
    /// secp256k1 ECDSA; public key is sent as SEC1 compressed 33 bytes; signature is raw 64-byte (r||s).
    EcdsaSecp256k1(k256::ecdsa::SigningKey),
}

impl SigningKeyEnum {
    /// Returns the canonical public key bytes to include in `AuthChallengeRequest.pubkey`.
    pub fn pubkey_bytes(&self) -> Vec<u8> {
        match self {
            SigningKeyEnum::Ed25519(k) => k.verifying_key().to_bytes().to_vec(),
            // 33-byte SEC1 compressed point — compact and natively supported by secp256k1 tooling
            SigningKeyEnum::EcdsaSecp256k1(k) => {
                k.verifying_key().to_encoded_point(true).as_bytes().to_vec()
            }
        }
    }

    /// Returns the proto `KeyType` discriminant to send in `AuthChallengeRequest.key_type`.
    pub fn proto_key_type(&self) -> ProtoKeyType {
        match self {
            SigningKeyEnum::Ed25519(_) => ProtoKeyType::Ed25519,
            SigningKeyEnum::EcdsaSecp256k1(_) => ProtoKeyType::EcdsaSecp256k1,
        }
    }

    /// Signs `msg` and returns raw signature bytes matching the server-side verification.
    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        match self {
            SigningKeyEnum::Ed25519(k) => {
                use ed25519_dalek::Signer as _;
                k.sign(msg).to_bytes().to_vec()
            }
            SigningKeyEnum::EcdsaSecp256k1(k) => {
                use k256::ecdsa::signature::Signer as _;
                let sig: k256::ecdsa::Signature = k.sign(msg);
                sig.to_bytes().to_vec()
            }
        }
    }
}

statemachine! {
    name: UserAgent,
    custom_error: false,
    transitions: {
        *Init + SentAuthChallengeRequest = WaitingForServerAuth,
        WaitingForServerAuth + ReceivedAuthChallenge = WaitingForAuthOk,
        WaitingForServerAuth + ReceivedAuthOk = Authenticated,
        WaitingForAuthOk + ReceivedAuthOk = Authenticated,
    }
}

pub struct DummyContext;
impl UserAgentStateMachineContext for DummyContext {}

#[derive(Debug, thiserror::Error)]
pub enum InboundError {
    #[error("Invalid user agent response")]
    InvalidResponse,
    #[error("Expected response payload")]
    MissingResponsePayload,
    #[error("Unexpected response payload")]
    UnexpectedResponsePayload,
    #[error("Invalid state for auth challenge")]
    InvalidStateForAuthChallenge,
    #[error("Invalid state for auth ok")]
    InvalidStateForAuthOk,
    #[error("State machine error")]
    StateTransitionFailed,
    #[error("Transport send failed")]
    TransportSendFailed,
}

pub struct UserAgentActor<Transport>
where
    Transport: Bi<UserAgentResponse, UserAgentRequest>,
{
    key: SigningKeyEnum,
    bootstrap_token: Option<String>,
    state: UserAgentStateMachine<DummyContext>,
    transport: Transport,
}

impl<Transport> UserAgentActor<Transport>
where
    Transport: Bi<UserAgentResponse, UserAgentRequest>,
{
    pub fn new(key: SigningKeyEnum, bootstrap_token: Option<String>, transport: Transport) -> Self {
        Self {
            key,
            bootstrap_token,
            state: UserAgentStateMachine::new(DummyContext),
            transport,
        }
    }

    fn transition(&mut self, event: UserAgentEvents) -> Result<(), InboundError> {
        self.state.process_event(event).map_err(|e| {
            error!(?e, "useragent state transition failed");
            InboundError::StateTransitionFailed
        })?;
        Ok(())
    }

    async fn send_auth_challenge_request(&mut self) -> Result<(), InboundError> {
        let req = AuthChallengeRequest {
            pubkey: self.key.pubkey_bytes(),
            bootstrap_token: self.bootstrap_token.take(),
            key_type: self.key.proto_key_type().into(),
        };

        self.transition(UserAgentEvents::SentAuthChallengeRequest)?;

        self.transport
            .send(UserAgentRequest {
                payload: Some(UserAgentRequestPayload::AuthChallengeRequest(req)),
            })
            .await
            .map_err(|_| InboundError::TransportSendFailed)?;

        info!(actor = "useragent", "auth.request.sent");
        Ok(())
    }

    async fn handle_auth_challenge(
        &mut self,
        challenge: arbiter_proto::proto::user_agent::AuthChallenge,
    ) -> Result<(), InboundError> {
        self.transition(UserAgentEvents::ReceivedAuthChallenge)?;

        let formatted = format_challenge(challenge.nonce, &challenge.pubkey);
        let signature_bytes = self.key.sign(&formatted);
        let solution = AuthChallengeSolution {
            signature: signature_bytes,
        };

        self.transport
            .send(UserAgentRequest {
                payload: Some(UserAgentRequestPayload::AuthChallengeSolution(solution)),
            })
            .await
            .map_err(|_| InboundError::TransportSendFailed)?;

        info!(actor = "useragent", "auth.solution.sent");
        Ok(())
    }

    fn handle_auth_ok(&mut self, _ok: AuthOk) -> Result<(), InboundError> {
        self.transition(UserAgentEvents::ReceivedAuthOk)?;
        info!(actor = "useragent", "auth.ok");
        Ok(())
    }

    pub async fn process_inbound_transport(
        &mut self,
        inbound: UserAgentResponse,
    ) -> Result<(), InboundError> {
        let payload = inbound
            .payload
            .ok_or(InboundError::MissingResponsePayload)?;

        match payload {
            UserAgentResponsePayload::AuthChallenge(challenge) => {
                self.handle_auth_challenge(challenge).await
            }
            UserAgentResponsePayload::AuthOk(ok) => self.handle_auth_ok(ok),
            _ => Err(InboundError::UnexpectedResponsePayload),
        }
    }
}

impl<Transport> Actor for UserAgentActor<Transport>
where
    Transport: Bi<UserAgentResponse, UserAgentRequest>,
{
    type Args = Self;

    type Error = ();

    async fn on_start(
        mut args: Self::Args,
        _actor_ref: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        if let Err(err) = args.send_auth_challenge_request().await {
            error!(?err, actor = "useragent", "auth.start.failed");
            return Err(());
        }
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
                inbound = self.transport.recv() => {
                    match inbound {
                        Some(inbound) => {
                            if let Err(err) = self.process_inbound_transport(inbound).await {
                                error!(?err, actor = "useragent", "transport.inbound.failed");
                                return Some(kameo::mailbox::Signal::Stop);
                            }
                        }
                        None => {
                            info!(actor = "useragent", "transport.closed");
                            return Some(kameo::mailbox::Signal::Stop);
                        }
                    }
                }
            }
        }
    }
}

mod grpc;
pub use grpc::{ConnectError, connect_grpc};
