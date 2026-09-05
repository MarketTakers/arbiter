use crate::{grpc::request_tracker::RequestTracker, peers::operator::auth};
use arbiter_crypto::authn;
use arbiter_proto::{
    proto::operator::{
        OperatorRequest, OperatorResponse,
        auth::{
            self as proto_auth, AuthChallenge as ProtoAuthChallenge,
            AuthChallengeRequest as ProtoAuthChallengeRequest,
            AuthChallengeSolution as ProtoAuthChallengeSolution, AuthResult as ProtoAuthResult,
            request::Payload as AuthRequestPayload, response::Payload as AuthResponsePayload,
        },
        operator_request::Payload as OperatorRequestPayload,
        operator_response::Payload as OperatorResponsePayload,
    },
    transport::{Bi, Error as TransportError, Receiver, Sender, grpc::GrpcBi},
};

use async_trait::async_trait;
use tonic::Status;
use tracing::warn;

pub(super) struct AuthTransportAdapter<'a> {
    pub(super) bi: &'a mut GrpcBi<OperatorRequest, OperatorResponse>,
    pub(super) request_tracker: &'a mut RequestTracker,
}

impl<'a> AuthTransportAdapter<'a> {
    pub(super) const fn new(
        bi: &'a mut GrpcBi<OperatorRequest, OperatorResponse>,
        request_tracker: &'a mut RequestTracker,
    ) -> Self {
        Self {
            bi,
            request_tracker,
        }
    }

    pub(super) const fn bi_mut(&mut self) -> &mut GrpcBi<OperatorRequest, OperatorResponse> {
        self.bi
    }

    pub(super) const fn tracker_mut(&mut self) -> &mut RequestTracker {
        self.request_tracker
    }

    pub(super) async fn send_response_payload(
        &mut self,
        payload: OperatorResponsePayload,
    ) -> Result<(), TransportError> {
        self.bi
            .send(Ok(OperatorResponse {
                id: Some(self.request_tracker.current_request_id()),
                payload: Some(payload),
            }))
            .await
    }

    async fn send_operator_response(
        &mut self,
        payload: AuthResponsePayload,
    ) -> Result<(), TransportError> {
        self.send_response_payload(OperatorResponsePayload::Auth(proto_auth::Response {
            payload: Some(payload),
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
            Ok(Outbound::AuthChallenge { challenge }) => {
                AuthResponsePayload::Challenge(ProtoAuthChallenge {
                    timestamp_nanos: challenge
                        .timestamp
                        .timestamp_nanos_opt()
                        .expect("timestamp within range")
                        .cast_unsigned(),
                    random: challenge.nonce.to_vec(),
                })
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

        self.send_operator_response(payload).await
    }
}

#[async_trait]
impl Receiver<auth::Inbound> for AuthTransportAdapter<'_> {
    async fn recv(&mut self) -> Option<auth::Inbound> {
        let request = match self.bi.recv().await? {
            Ok(request) => request,
            Err(error) => {
                warn!(error = ?error, "Failed to receive operator auth request");
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
                "grpc.operator.auth_adapter"
            );
            return None;
        };

        let OperatorRequestPayload::Auth(auth_request) = payload else {
            let _ = self
                .bi
                .send(Err(Status::invalid_argument(
                    "Unsupported operator auth request",
                )))
                .await;
            return None;
        };

        let Some(payload) = auth_request.payload else {
            warn!(
                event = "received auth request with empty payload",
                "grpc.operator.auth_adapter"
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
                        "grpc.operator.auth_adapter"
                    );
                    return None;
                };

                Some(auth::Inbound::AuthChallengeRequest {
                    pubkey,
                    bootstrap_token: bootstrap_token.map(String::into_bytes),
                })
            }
            AuthRequestPayload::ChallengeSolution(ProtoAuthChallengeSolution { signature }) => {
                Some(auth::Inbound::AuthChallengeSolution { signature })
            }
        }
    }
}

impl Bi<auth::Inbound, Result<auth::Outbound, auth::Error>> for AuthTransportAdapter<'_> {}
