use arbiter_crypto::authn;
use arbiter_proto::{
    proto::user_agent::{
        UserAgentRequest, UserAgentResponse,
        auth::{
            self as proto_auth, AuthChallenge as ProtoAuthChallenge,
            AuthChallengeRequest as ProtoAuthChallengeRequest,
            AuthChallengeSolution as ProtoAuthChallengeSolution, AuthResult as ProtoAuthResult,
            request::Payload as AuthRequestPayload, response::Payload as AuthResponsePayload,
        },
        user_agent_request::Payload as UserAgentRequestPayload,
        user_agent_response::Payload as UserAgentResponsePayload,
    },
    transport::{Bi, Error as TransportError, Receiver, Sender, grpc::GrpcBi},
};
use async_trait::async_trait;
use tonic::Status;
use tracing::warn;

use crate::{
    grpc::request_tracker::RequestTracker,
    peers::user_agent::{UserAgentConnection, auth},
};

pub struct AuthTransportAdapter<'a> {
    bi: &'a mut GrpcBi<UserAgentRequest, UserAgentResponse>,
    request_tracker: &'a mut RequestTracker,
}

impl<'a> AuthTransportAdapter<'a> {
    pub fn new(
        bi: &'a mut GrpcBi<UserAgentRequest, UserAgentResponse>,
        request_tracker: &'a mut RequestTracker,
    ) -> Self {
        Self {
            bi,
            request_tracker,
        }
    }

    async fn send_user_agent_response(
        &mut self,
        payload: AuthResponsePayload,
    ) -> Result<(), TransportError> {
        self.bi
            .send(Ok(UserAgentResponse {
                id: Some(self.request_tracker.current_request_id()),
                payload: Some(UserAgentResponsePayload::Auth(proto_auth::Response {
                    payload: Some(payload),
                })),
            }))
            .await
    }
}

#[async_trait]
impl Sender<Result<auth::Outbound, auth::Error>> for AuthTransportAdapter<'_> {
    async fn send(
        &mut self,
        item: Result<auth::Outbound, auth::Error>,
    ) -> Result<(), TransportError> {
        use auth::{Error, Outbound};
        let payload = match item {
            Ok(Outbound::AuthChallenge { nonce }) => {
                AuthResponsePayload::Challenge(ProtoAuthChallenge { nonce })
            }
            Ok(Outbound::AuthSuccess) => {
                AuthResponsePayload::Result(ProtoAuthResult::Success.into())
            }
            Err(Error::UnregisteredPublicKey) => {
                AuthResponsePayload::Result(ProtoAuthResult::InvalidKey.into())
            }
            Err(Error::InvalidChallengeSolution) => {
                AuthResponsePayload::Result(ProtoAuthResult::InvalidSignature.into())
            }
            Err(Error::InvalidBootstrapToken) => {
                AuthResponsePayload::Result(ProtoAuthResult::TokenInvalid.into())
            }
            Err(Error::Internal { details }) => {
                return self.bi.send(Err(Status::internal(details))).await;
            }
            Err(Error::Transport) => {
                return self
                    .bi
                    .send(Err(Status::unavailable("transport error")))
                    .await;
            }
        };

        self.send_user_agent_response(payload).await
    }
}

#[async_trait]
impl Receiver<auth::Inbound> for AuthTransportAdapter<'_> {
    async fn recv(&mut self) -> Option<auth::Inbound> {
        let request = match self.bi.recv().await? {
            Ok(request) => request,
            Err(error) => {
                warn!(error = ?error, "Failed to receive user agent auth request");
                return None;
            }
        };

        match self.request_tracker.request(request.id) {
            Ok(request_id) => request_id,
            Err(error) => {
                let _ = self.bi.send(Err(error)).await;
                return None;
            }
        };

        let Some(payload) = request.payload else {
            warn!(
                event = "received request with empty payload",
                "grpc.useragent.auth_adapter"
            );
            return None;
        };

        let UserAgentRequestPayload::Auth(auth_request) = payload else {
            let _ = self
                .bi
                .send(Err(Status::invalid_argument(
                    "Unsupported user-agent auth request",
                )))
                .await;
            return None;
        };

        let Some(payload) = auth_request.payload else {
            warn!(
                event = "received auth request with empty payload",
                "grpc.useragent.auth_adapter"
            );
            return None;
        };

        match payload {
            AuthRequestPayload::ChallengeRequest(ProtoAuthChallengeRequest {
                pubkey,
                bootstrap_token,
            }) => {
                let Ok(pubkey) = authn::PublicKey::try_from(pubkey.as_slice()) else {
                    warn!(
                        event = "received request with invalid public key",
                        "grpc.useragent.auth_adapter"
                    );
                    return None;
                };

                Some(auth::Inbound::AuthChallengeRequest {
                    pubkey,
                    bootstrap_token,
                })
            }
            AuthRequestPayload::ChallengeSolution(ProtoAuthChallengeSolution { signature }) => {
                Some(auth::Inbound::AuthChallengeSolution { signature })
            }
        }
    }
}

impl Bi<auth::Inbound, Result<auth::Outbound, auth::Error>> for AuthTransportAdapter<'_> {}

pub async fn start(
    conn: &mut UserAgentConnection,
    bi: &mut GrpcBi<UserAgentRequest, UserAgentResponse>,
    request_tracker: &mut RequestTracker,
) -> Result<(i32, authn::PublicKey), auth::Error> {
    let transport = AuthTransportAdapter::new(bi, request_tracker);
    auth::authenticate(conn, transport).await
}
