use arbiter_proto::{
    proto::client::{
        AuthChallenge as ProtoAuthChallenge, AuthChallengeRequest as ProtoAuthChallengeRequest,
        AuthChallengeSolution as ProtoAuthChallengeSolution, AuthResult as ProtoAuthResult,
        ClientRequest, ClientResponse, client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
    },
    transport::{Bi, Error as TransportError, Receiver, Sender, grpc::GrpcBi},
};
use async_trait::async_trait;
use tracing::warn;

use crate::actors::client::{self, ClientConnection, auth};

pub struct AuthTransportAdapter<'a>(pub(super) &'a mut GrpcBi<ClientRequest, ClientResponse>);

impl AuthTransportAdapter<'_> {
    fn response_to_proto(response: auth::Outbound) -> ClientResponse {
        let payload = match response {
            auth::Outbound::AuthChallenge { pubkey, nonce } => {
                ClientResponsePayload::AuthChallenge(ProtoAuthChallenge {
                    pubkey: pubkey.to_bytes().to_vec(),
                    nonce,
                })
            }
            auth::Outbound::AuthSuccess => {
                ClientResponsePayload::AuthResult(ProtoAuthResult::Success.into())
            }
        };

        ClientResponse {
            payload: Some(payload),
        }
    }

    fn error_to_proto(error: auth::Error) -> ClientResponse {
        ClientResponse {
            payload: Some(ClientResponsePayload::AuthResult(
                match error {
                    auth::Error::InvalidChallengeSolution => ProtoAuthResult::InvalidSignature,
                    auth::Error::ApproveError(auth::ApproveError::Denied) => {
                        ProtoAuthResult::ApprovalDenied
                    }
                    auth::Error::ApproveError(auth::ApproveError::Upstream(
                        crate::actors::router::ApprovalError::NoUserAgentsConnected,
                    )) => ProtoAuthResult::NoUserAgentsOnline,
                    auth::Error::ApproveError(auth::ApproveError::Internal)
                    | auth::Error::DatabasePoolUnavailable
                    | auth::Error::DatabaseOperationFailed
                    | auth::Error::Transport => ProtoAuthResult::Internal,
                }
                .into(),
            )),
        }
    }

    async fn send_auth_result(&mut self, result: ProtoAuthResult) -> Result<(), TransportError> {
        self.0
            .send(Ok(ClientResponse {
                payload: Some(ClientResponsePayload::AuthResult(result.into())),
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
        let outbound = match item {
            Ok(message) => Ok(AuthTransportAdapter::response_to_proto(message)),
            Err(err) => Ok(AuthTransportAdapter::error_to_proto(err)),
        };

        self.0.send(outbound).await
    }
}

#[async_trait]
impl Receiver<auth::Inbound> for AuthTransportAdapter<'_> {
    async fn recv(&mut self) -> Option<auth::Inbound> {
        let request = match self.0.recv().await? {
            Ok(request) => request,
            Err(error) => {
                warn!(error = ?error, "grpc client recv failed; closing stream");
                return None;
            }
        };

        let payload = request.payload?;

        match payload {
            ClientRequestPayload::AuthChallengeRequest(ProtoAuthChallengeRequest { pubkey }) => {
                let Ok(pubkey) = <[u8; 32]>::try_from(pubkey) else {
                    let _ = self.send_auth_result(ProtoAuthResult::InvalidKey).await;
                    return None;
                };
                let Ok(pubkey) = ed25519_dalek::VerifyingKey::from_bytes(&pubkey) else {
                    let _ = self.send_auth_result(ProtoAuthResult::InvalidKey).await;
                    return None;
                };
                Some(auth::Inbound::AuthChallengeRequest { pubkey })
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
            _ => None,
        }
    }
}

impl Bi<auth::Inbound, Result<auth::Outbound, auth::Error>> for AuthTransportAdapter<'_> {}

pub async fn start(
    conn: &mut ClientConnection,
    bi: &mut GrpcBi<ClientRequest, ClientResponse>,
) -> Result<(), auth::Error> {
    let mut transport = AuthTransportAdapter(bi);
    client::auth::authenticate(conn, &mut transport).await?;
    Ok(())
}
