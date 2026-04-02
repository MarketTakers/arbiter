use arbiter_proto::{
    ClientMetadata,
    proto::client::{
        AuthChallenge as ProtoAuthChallenge, AuthChallengeRequest as ProtoAuthChallengeRequest,
        AuthChallengeSolution as ProtoAuthChallengeSolution, AuthResult as ProtoAuthResult,
        ClientInfo as ProtoClientInfo, ClientRequest, ClientResponse,
        client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
    },
    transport::{Bi, Error as TransportError, Receiver, Sender, grpc::GrpcBi},
};
use async_trait::async_trait;
use tonic::Status;
use tracing::warn;

use crate::{
    actors::client::{self, ClientConnection, auth},
    grpc::request_tracker::RequestTracker,
};

pub struct AuthTransportAdapter<'a> {
    bi: &'a mut GrpcBi<ClientRequest, ClientResponse>,
    request_tracker: &'a mut RequestTracker,
}

impl<'a> AuthTransportAdapter<'a> {
    pub fn new(
        bi: &'a mut GrpcBi<ClientRequest, ClientResponse>,
        request_tracker: &'a mut RequestTracker,
    ) -> Self {
        Self {
            bi,
            request_tracker,
        }
    }

    fn response_to_proto(response: auth::Outbound) -> ClientResponsePayload {
        match response {
            auth::Outbound::AuthChallenge { pubkey, nonce } => {
                ClientResponsePayload::AuthChallenge(ProtoAuthChallenge {
                    pubkey: pubkey.to_bytes().to_vec(),
                    nonce,
                })
            }
            auth::Outbound::AuthSuccess => {
                ClientResponsePayload::AuthResult(ProtoAuthResult::Success.into())
            }
        }
    }

    fn error_to_proto(error: auth::Error) -> ClientResponsePayload {
        ClientResponsePayload::AuthResult(
            match error {
                auth::Error::InvalidChallengeSolution => ProtoAuthResult::InvalidSignature,
                auth::Error::ApproveError(auth::ApproveError::Denied) => {
                    ProtoAuthResult::ApprovalDenied
                }
                auth::Error::ApproveError(auth::ApproveError::Upstream(
                    crate::actors::flow_coordinator::ApprovalError::NoUserAgentsConnected,
                )) => ProtoAuthResult::NoUserAgentsOnline,
                auth::Error::ApproveError(auth::ApproveError::Internal)
                | auth::Error::DatabasePoolUnavailable
                | auth::Error::DatabaseOperationFailed
                | auth::Error::Transport => ProtoAuthResult::Internal,
            }
            .into(),
        )
    }

    async fn send_client_response(
        &mut self,
        payload: ClientResponsePayload,
    ) -> Result<(), TransportError> {
        self.bi
            .send(Ok(ClientResponse {
                request_id: Some(self.request_tracker.current_request_id()),
                payload: Some(payload),
            }))
            .await
    }

    async fn send_auth_result(&mut self, result: ProtoAuthResult) -> Result<(), TransportError> {
        self.send_client_response(ClientResponsePayload::AuthResult(result.into()))
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
            Ok(message) => AuthTransportAdapter::response_to_proto(message),
            Err(err) => AuthTransportAdapter::error_to_proto(err),
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

        match payload {
            ClientRequestPayload::AuthChallengeRequest(ProtoAuthChallengeRequest {
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
                let Ok(pubkey) = <[u8; 32]>::try_from(pubkey) else {
                    let _ = self.send_auth_result(ProtoAuthResult::InvalidKey).await;
                    return None;
                };
                let Ok(pubkey) = ed25519_dalek::VerifyingKey::from_bytes(&pubkey) else {
                    let _ = self.send_auth_result(ProtoAuthResult::InvalidKey).await;
                    return None;
                };
                Some(auth::Inbound::AuthChallengeRequest {
                    pubkey,
                    metadata: client_metadata_from_proto(client_info),
                })
            }
            ClientRequestPayload::AuthChallengeSolution(ProtoAuthChallengeSolution {
                signature,
            }) => {
                let Ok(signature) = ed25519_dalek::Signature::try_from(signature.as_slice()) else {
                    let _ = self
                        .send_auth_result(ProtoAuthResult::InvalidSignature)
                        .await;
                    return None;
                };
                Some(auth::Inbound::AuthChallengeSolution { signature })
            }
            _ => {
                let _ = self
                    .bi
                    .send(Err(Status::invalid_argument(
                        "Unsupported client auth request",
                    )))
                    .await;
                None
            }
        }
    }
}

impl Bi<auth::Inbound, Result<auth::Outbound, auth::Error>> for AuthTransportAdapter<'_> {}

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
) -> Result<i32, auth::Error> {
    let mut transport = AuthTransportAdapter::new(bi, request_tracker);
    client::auth::authenticate(conn, &mut transport).await
}
