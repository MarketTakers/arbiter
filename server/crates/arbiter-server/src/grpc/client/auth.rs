use arbiter_crypto::authn;
use arbiter_proto::{
    ClientMetadata,
    proto::{
        client::{
            ClientRequest, ClientResponse,
            auth::{
                self as proto_auth, AuthChallengeRequest as ProtoAuthChallengeRequest,
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

use crate::{
    actors::client::{ClientConnection, auth},
    grpc::request_tracker::RequestTracker,
};

pub struct AuthTransportAdapter<'a> {
    bi: &'a mut GrpcBi<ClientRequest, ClientResponse>,
    request_tracker: &'a mut RequestTracker,
}

impl<'a> AuthTransportAdapter<'a> {
    pub const fn new(
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
impl Sender<Result<auth::Outbound, auth::ClientAuthError>> for AuthTransportAdapter<'_> {
    async fn send(
        &mut self,
        item: Result<auth::Outbound, auth::ClientAuthError>,
    ) -> Result<(), TransportError> {
        let payload = match item {
            Ok(message) => message.into(),
            Err(err) => AuthResponsePayload::Result(ProtoAuthResult::from(err).into()),
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
                    metadata: client_metadata_from_proto(client_info),
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

impl Bi<auth::Inbound, Result<auth::Outbound, auth::ClientAuthError>> for AuthTransportAdapter<'_> {}

fn client_metadata_from_proto(metadata: ProtoClientInfo) -> ClientMetadata {
    ClientMetadata {
        name: metadata.name,
        description: metadata.description,
        version: metadata.version,
    }
}

pub async fn start(
    conn: &mut ClientConnection,
    bi: &mut GrpcBi<ClientRequest, ClientResponse>,
    request_tracker: &mut RequestTracker,
) -> Result<i32, auth::ClientAuthError> {
    let mut transport = AuthTransportAdapter::new(bi, request_tracker);
    auth::authenticate(conn, &mut transport).await
}
