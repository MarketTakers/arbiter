use alloy::{
    consensus::SignableTransaction,
    network::TxSigner,
    primitives::{Address, B256, ChainId, Signature},
    signers::{Error, Result, Signer},
};
use arbiter_proto::{
    format_challenge,
    proto::{
        arbiter_service_client::ArbiterServiceClient,
        client::{
            AuthChallengeRequest, AuthChallengeSolution, ClientRequest, ClientResponse,
            client_connect_error, client_request::Payload as ClientRequestPayload,
            client_response::Payload as ClientResponsePayload,
        },
        evm::{
            EvmSignTransactionRequest, evm_sign_transaction_response::Result as SignResponseResult,
        },
    },
    url::ArbiterUrl,
};
use async_trait::async_trait;
use ed25519_dalek::Signer as _;
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::ClientTlsConfig;

const BUFFER_LENGTH: usize = 16;

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("Could not establish connection")]
    Connection(#[from] tonic::transport::Error),

    #[error("Invalid server URI")]
    InvalidUri(#[from] http::uri::InvalidUri),

    #[error("Invalid CA certificate")]
    InvalidCaCert(#[from] webpki::Error),

    #[error("gRPC error")]
    Grpc(#[from] tonic::Status),

    #[error("Auth challenge was not returned by server")]
    MissingAuthChallenge,

    #[error("Client approval denied by User Agent")]
    ApprovalDenied,

    #[error("No User Agents online to approve client")]
    NoUserAgentsOnline,

    #[error("Unexpected auth response payload")]
    UnexpectedAuthResponse,
}

#[derive(Debug, thiserror::Error)]
enum ClientSignError {
    #[error("Transport channel closed")]
    ChannelClosed,

    #[error("Connection closed by server")]
    ConnectionClosed,

    #[error("Invalid response payload")]
    InvalidResponse,

    #[error("Remote signing was rejected")]
    Rejected,
}

struct ClientTransport {
    sender: mpsc::Sender<ClientRequest>,
    receiver: tonic::Streaming<ClientResponse>,
}

impl ClientTransport {
    async fn send(&mut self, request: ClientRequest) -> std::result::Result<(), ClientSignError> {
        self.sender
            .send(request)
            .await
            .map_err(|_| ClientSignError::ChannelClosed)
    }

    async fn recv(&mut self) -> std::result::Result<ClientResponse, ClientSignError> {
        match self.receiver.message().await {
            Ok(Some(resp)) => Ok(resp),
            Ok(None) => Err(ClientSignError::ConnectionClosed),
            Err(_) => Err(ClientSignError::ConnectionClosed),
        }
    }
}

pub struct ArbiterSigner {
    transport: Mutex<ClientTransport>,
    address: Address,
    chain_id: Option<ChainId>,
}

impl ArbiterSigner {
    pub async fn connect_grpc(
        url: ArbiterUrl,
        key: ed25519_dalek::SigningKey,
        address: Address,
    ) -> std::result::Result<Self, ConnectError> {
        let anchor = webpki::anchor_from_trusted_cert(&url.ca_cert)?.to_owned();
        let tls = ClientTlsConfig::new().trust_anchor(anchor);

        // NOTE: We intentionally keep the same URL construction strategy as the user-agent crate
        // to avoid behavior drift between the two clients.
        let channel = tonic::transport::Channel::from_shared(format!("{}:{}", url.host, url.port))?
            .tls_config(tls)?
            .connect()
            .await?;

        let mut client = ArbiterServiceClient::new(channel);
        let (tx, rx) = mpsc::channel(BUFFER_LENGTH);
        let response_stream = client.client(ReceiverStream::new(rx)).await?.into_inner();

        let mut transport = ClientTransport {
            sender: tx,
            receiver: response_stream,
        };

        authenticate(&mut transport, key).await?;

        Ok(Self {
            transport: Mutex::new(transport),
            address,
            chain_id: None,
        })
    }

    async fn sign_transaction_via_arbiter(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> Result<Signature> {
        if let Some(chain_id) = self.chain_id
            && !tx.set_chain_id_checked(chain_id)
        {
            return Err(Error::TransactionChainIdMismatch {
                signer: chain_id,
                tx: tx.chain_id().unwrap(),
            });
        }

        let mut rlp_transaction = Vec::new();
        tx.encode_for_signing(&mut rlp_transaction);

        let request = ClientRequest {
            payload: Some(ClientRequestPayload::EvmSignTransaction(
                EvmSignTransactionRequest {
                    wallet_address: self.address.as_slice().to_vec(),
                    rlp_transaction,
                },
            )),
        };

        let mut transport = self.transport.lock().await;
        transport.send(request).await.map_err(Error::other)?;
        let response = transport.recv().await.map_err(Error::other)?;

        let payload = response
            .payload
            .ok_or_else(|| Error::other(ClientSignError::InvalidResponse))?;

        let ClientResponsePayload::EvmSignTransaction(sign_response) = payload else {
            return Err(Error::other(ClientSignError::InvalidResponse));
        };

        let Some(result) = sign_response.result else {
            return Err(Error::other(ClientSignError::InvalidResponse));
        };

        match result {
            SignResponseResult::Signature(bytes) => {
                Signature::try_from(bytes.as_slice()).map_err(Error::other)
            }
            SignResponseResult::EvalError(_) | SignResponseResult::Error(_) => {
                Err(Error::other(ClientSignError::Rejected))
            }
        }
    }
}

async fn authenticate(
    transport: &mut ClientTransport,
    key: ed25519_dalek::SigningKey,
) -> std::result::Result<(), ConnectError> {
    transport
        .send(ClientRequest {
            payload: Some(ClientRequestPayload::AuthChallengeRequest(
                AuthChallengeRequest {
                    pubkey: key.verifying_key().to_bytes().to_vec(),
                },
            )),
        })
        .await
        .map_err(|_| ConnectError::UnexpectedAuthResponse)?;

    let response = transport
        .recv()
        .await
        .map_err(|_| ConnectError::MissingAuthChallenge)?;

    let payload = response.payload.ok_or(ConnectError::MissingAuthChallenge)?;
    match payload {
        ClientResponsePayload::AuthChallenge(challenge) => {
            let challenge_payload = format_challenge(challenge.nonce, &challenge.pubkey);
            let signature = key.sign(&challenge_payload).to_bytes().to_vec();

            transport
                .send(ClientRequest {
                    payload: Some(ClientRequestPayload::AuthChallengeSolution(
                        AuthChallengeSolution { signature },
                    )),
                })
                .await
                .map_err(|_| ConnectError::UnexpectedAuthResponse)?;

            // Current server flow does not emit `AuthOk` for SDK clients, so we proceed after
            // sending the solution. If authentication fails, the first business request will return
            // a `ClientConnectError` or the stream will close.
            Ok(())
        }
        ClientResponsePayload::ClientConnectError(err) => {
            match client_connect_error::Code::try_from(err.code)
                .unwrap_or(client_connect_error::Code::Unknown)
            {
                client_connect_error::Code::ApprovalDenied => Err(ConnectError::ApprovalDenied),
                client_connect_error::Code::NoUserAgentsOnline => {
                    Err(ConnectError::NoUserAgentsOnline)
                }
                client_connect_error::Code::Unknown => Err(ConnectError::UnexpectedAuthResponse),
            }
        }
        _ => Err(ConnectError::UnexpectedAuthResponse),
    }
}

#[async_trait]
impl Signer for ArbiterSigner {
    async fn sign_hash(&self, _hash: &B256) -> Result<Signature> {
        Err(Error::other(
            "hash-only signing is not supported for ArbiterSigner; use transaction signing",
        ))
    }

    fn address(&self) -> Address {
        self.address
    }

    fn chain_id(&self) -> Option<ChainId> {
        self.chain_id
    }

    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        self.chain_id = chain_id;
    }
}

#[async_trait]
impl TxSigner<Signature> for ArbiterSigner {
    fn address(&self) -> Address {
        self.address
    }

    async fn sign_transaction(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> Result<Signature> {
        self.sign_transaction_via_arbiter(tx).await
    }
}
