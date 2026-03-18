use arbiter_proto::{
    proto::client::{
        AuthChallenge as ProtoAuthChallenge,
        AuthChallengeRequest as ProtoAuthChallengeRequest,
        AuthChallengeSolution as ProtoAuthChallengeSolution, AuthOk as ProtoAuthOk,
        ClientConnectError, ClientRequest, ClientResponse,
        client_connect_error::Code as ProtoClientConnectErrorCode,
        client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
    },
    transport::{Bi, Error as TransportError},
};
use async_trait::async_trait;
use futures::StreamExt as _;
use tokio::sync::mpsc;
use tonic::{Status, Streaming};

use crate::actors::client::{
    self, ClientError, ConnectErrorCode, Request as DomainRequest, Response as DomainResponse,
};

pub struct GrpcTransport {
    sender: mpsc::Sender<Result<ClientResponse, Status>>,
    receiver: Streaming<ClientRequest>,
}

impl GrpcTransport {
    pub fn new(
        sender: mpsc::Sender<Result<ClientResponse, Status>>,
        receiver: Streaming<ClientRequest>,
    ) -> Self {
        Self { sender, receiver }
    }

    fn request_to_domain(request: ClientRequest) -> Result<DomainRequest, Status> {
        match request.payload {
            Some(ClientRequestPayload::AuthChallengeRequest(ProtoAuthChallengeRequest {
                pubkey,
            })) => Ok(DomainRequest::AuthChallengeRequest { pubkey }),
            Some(ClientRequestPayload::AuthChallengeSolution(
                ProtoAuthChallengeSolution { signature },
            )) => Ok(DomainRequest::AuthChallengeSolution { signature }),
            None => Err(Status::invalid_argument("Missing client request payload")),
        }
    }

    fn response_to_proto(response: DomainResponse) -> ClientResponse {
        let payload = match response {
            DomainResponse::AuthChallenge { pubkey, nonce } => {
                ClientResponsePayload::AuthChallenge(ProtoAuthChallenge { pubkey, nonce })
            }
            DomainResponse::AuthOk => ClientResponsePayload::AuthOk(ProtoAuthOk {}),
            DomainResponse::ClientConnectError { code } => {
                ClientResponsePayload::ClientConnectError(ClientConnectError {
                    code: match code {
                        ConnectErrorCode::Unknown => ProtoClientConnectErrorCode::Unknown,
                        ConnectErrorCode::ApprovalDenied => {
                            ProtoClientConnectErrorCode::ApprovalDenied
                        }
                        ConnectErrorCode::NoUserAgentsOnline => {
                            ProtoClientConnectErrorCode::NoUserAgentsOnline
                        }
                    }
                    .into(),
                })
            }
        };

        ClientResponse {
            payload: Some(payload),
        }
    }

    fn error_to_status(value: ClientError) -> Status {
        match value {
            ClientError::MissingRequestPayload | ClientError::UnexpectedRequestPayload => {
                Status::invalid_argument("Expected message with payload")
            }
            ClientError::StateTransitionFailed => Status::internal("State machine error"),
            ClientError::Auth(ref err) => auth_error_status(err),
            ClientError::ConnectionRegistrationFailed => {
                Status::internal("Connection registration failed")
            }
        }
    }
}

#[async_trait]
impl Bi<DomainRequest, Result<DomainResponse, ClientError>> for GrpcTransport {
    async fn send(&mut self, item: Result<DomainResponse, ClientError>) -> Result<(), TransportError> {
        let outbound = match item {
            Ok(message) => Ok(Self::response_to_proto(message)),
            Err(err) => Err(Self::error_to_status(err)),
        };

        self.sender
            .send(outbound)
            .await
            .map_err(|_| TransportError::ChannelClosed)
    }

    async fn recv(&mut self) -> Option<DomainRequest> {
        match self.receiver.next().await {
            Some(Ok(item)) => match Self::request_to_domain(item) {
                Ok(request) => Some(request),
                Err(status) => {
                    let _ = self.sender.send(Err(status)).await;
                    None
                }
            },
            Some(Err(error)) => {
                tracing::error!(error = ?error, "grpc client recv failed; closing stream");
                None
            }
            None => None,
        }
    }
}

fn auth_error_status(value: &client::auth::Error) -> Status {
    use client::auth::Error;

    match value {
        Error::UnexpectedMessagePayload | Error::InvalidClientPubkeyLength => {
            Status::invalid_argument(value.to_string())
        }
        Error::InvalidAuthPubkeyEncoding => {
            Status::invalid_argument("Failed to convert pubkey to VerifyingKey")
        }
        Error::InvalidChallengeSolution => Status::unauthenticated(value.to_string()),
        Error::ApproveError(_) => Status::permission_denied(value.to_string()),
        Error::Transport => Status::internal("Transport error"),
        Error::DatabasePoolUnavailable => Status::internal("Database pool error"),
        Error::DatabaseOperationFailed => Status::internal("Database error"),
        Error::InternalError => Status::internal("Internal error"),
    }
}
