use crate::{
    grpc::{Convert, request_tracker::RequestTracker},
    peers::client::{ClientConnection, auth},
};
use arbiter_crypto::authn;
use arbiter_proto::{
    ClientMetadata,
    proto::{
        client::{
            ClientRequest, ClientResponse,
            auth::{
                self as proto_auth, AuthChallenge as ProtoAuthChallenge,
                AuthChallengeRequest as ProtoAuthChallengeRequest,
                AuthChallengeSolution as ProtoAuthChallengeSolution, AuthResult as ProtoAuthResult,
                request::Payload as AuthRequestPayload, response::Payload as AuthResponsePayload,
            },
            client_request::Payload as ClientRequestPayload,
            client_response::Payload as ClientResponsePayload,
        },
        shared::ClientInfo as ProtoClientInfo,
    },
    transport::{Bi, Error as TransportError, Receiver, Sender, grpc::GrpcBi},
};

use async_trait::async_trait;
use tonic::Status;
use tracing::warn;

pub(super) struct AuthTransportAdapter<'a> {
    bi: &'a mut GrpcBi<ClientRequest, ClientResponse>,
    request_tracker: &'a mut RequestTracker,
}

impl<'a> AuthTransportAdapter<'a> {
    pub(super) const fn new(
        bi: &'a mut GrpcBi<ClientRequest, ClientResponse>,
        request_tracker: &'a mut RequestTracker,
    ) -> Self {
        Self {
            bi,
            request_tracker,
        }
    }

    async fn send_client_response(
        &mut self,
        payload: AuthResponsePayload,
    ) -> Result<(), TransportError> {
        self.bi
            .send(Ok(ClientResponse {
                request_id: Some(self.request_tracker.current_request_id()),
                payload: Some(ClientResponsePayload::Auth(proto_auth::Response {
                    payload: Some(payload),
                })),
            }))
            .await
    }

    async fn send_auth_result(&mut self, result: ProtoAuthResult) -> Result<(), TransportError> {
        self.send_client_response(AuthResponsePayload::Result(result.into()))
            .await
    }
}

#[async_trait]
impl Sender<Result<auth::Outbound, auth::Error>> for AuthTransportAdapter<'_> {
    async fn send(
        &mut self,
        item: Result<auth::Outbound, auth::Error>,
    ) -> Result<(), TransportError> {
        let payload = match item {
            Ok(message) => message.convert(),
            Err(err) => err.convert(),
        };

        self.send_client_response(payload).await
    }
}

#[async_trait]
impl Receiver<auth::Inbound> for AuthTransportAdapter<'_> {
    async fn recv(&mut self) -> Option<auth::Inbound> {
        let request = match self.bi.recv().await? {
            Ok(request) => request,
            Err(error) => {
                warn!(error = ?error, "grpc client recv failed; closing stream");
                return None;
            }
        };

        match self.request_tracker.request(request.request_id) {
            Ok(request_id) => request_id,
            Err(error) => {
                let _ = self.bi.send(Err(error)).await;
                return None;
            }
        };
        let payload = request.payload?;
        let ClientRequestPayload::Auth(auth_request) = payload else {
            let _ = self
                .bi
                .send(Err(Status::invalid_argument(
                    "Unsupported client auth request",
                )))
                .await;
            return None;
        };
        let Some(payload) = auth_request.payload else {
            let _ = self
                .bi
                .send(Err(Status::invalid_argument(
                    "Missing client auth request payload",
                )))
                .await;
            return None;
        };

        match payload {
            AuthRequestPayload::ChallengeRequest(ProtoAuthChallengeRequest {
                pubkey,
                client_info,
            }) => {
                let Some(client_info) = client_info else {
                    let _ = self
                        .bi
                        .send(Err(Status::invalid_argument("Missing client info")))
                        .await;
                    return None;
                };
                let Ok(pubkey) = authn::PublicKey::try_from(pubkey.as_slice()) else {
                    let _ = self.send_auth_result(ProtoAuthResult::InvalidKey).await;
                    return None;
                };
                Some(auth::Inbound::AuthChallengeRequest {
                    pubkey,
                    metadata: client_info.convert(),
                })
            }
            AuthRequestPayload::ChallengeSolution(ProtoAuthChallengeSolution { signature }) => {
                let Ok(signature) = authn::Signature::try_from(signature.as_slice()) else {
                    let _ = self
                        .send_auth_result(ProtoAuthResult::InvalidSignature)
                        .await;
                    return None;
                };
                Some(auth::Inbound::AuthChallengeSolution { signature })
            }
        }
    }
}

impl Bi<auth::Inbound, Result<auth::Outbound, auth::Error>> for AuthTransportAdapter<'_> {}

impl Convert for ProtoClientInfo {
    type Output = ClientMetadata;

    fn convert(self) -> Self::Output {
        ClientMetadata {
            name: self.name,
            description: self.description,
            version: self.version,
        }
    }
}

impl Convert for auth::Error {
    type Output = AuthResponsePayload;

    fn convert(self) -> Self::Output {
        use auth::Error::{
            ApproveError, DatabaseOperationFailed, DatabasePoolUnavailable, IntegrityCheckFailed,
            InvalidChallengeSolution, Transport,
        };
        AuthResponsePayload::Result(
            match self {
                InvalidChallengeSolution => ProtoAuthResult::InvalidSignature,
                ApproveError(auth::ApproveError::Denied) => ProtoAuthResult::ApprovalDenied,
                ApproveError(auth::ApproveError::Upstream(
                    crate::actors::flow_coordinator::ApprovalError::NoOperatorsConnected,
                )) => ProtoAuthResult::NoOperatorsOnline,
                ApproveError(auth::ApproveError::Internal)
                | DatabasePoolUnavailable
                | DatabaseOperationFailed
                | IntegrityCheckFailed
                | Transport => ProtoAuthResult::Internal,
            }
            .into(),
        )
    }
}

impl Convert for auth::Outbound {
    type Output = AuthResponsePayload;

    fn convert(self) -> Self::Output {
        match self {
            Self::AuthChallenge { challenge } => {
                AuthResponsePayload::Challenge(ProtoAuthChallenge {
                    timestamp_nanos: challenge
                        .timestamp
                        .timestamp_nanos_opt()
                        .expect("timestamp within range")
                        as u64,
                    random: challenge.nonce.to_vec(),
                })
            }
            Self::AuthSuccess => AuthResponsePayload::Result(ProtoAuthResult::Success.into()),
        }
    }
}

pub(super) async fn start(
    conn: &mut ClientConnection,
    bi: &mut GrpcBi<ClientRequest, ClientResponse>,
    request_tracker: &mut RequestTracker,
) -> Result<i32, auth::Error> {
    let mut transport = AuthTransportAdapter::new(bi, request_tracker);
    auth::authenticate(conn, &mut transport).await
}
