use alloy::{
    consensus::SignableTransaction,
    network::TxSigner,
    primitives::{Address, B256, ChainId, Signature},
    signers::{Error, Result, Signer},
};
use arbiter_proto::{
    format_challenge, home_path,
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
use std::path::{Path, PathBuf};
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

    #[error("Signing key storage error")]
    Storage(#[from] StorageError),
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("Invalid signing key length in storage: expected {expected} bytes, got {actual} bytes")]
    InvalidKeyLength { expected: usize, actual: usize },
}

pub trait SigningKeyStorage {
    fn load_or_create(&self) -> std::result::Result<ed25519_dalek::SigningKey, StorageError>;
}

#[derive(Debug, Clone)]
pub struct FileSigningKeyStorage {
    path: PathBuf,
}

impl FileSigningKeyStorage {
    pub const DEFAULT_FILE_NAME: &str = "sdk_client_ed25519.key";

    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn from_default_location() -> std::result::Result<Self, StorageError> {
        Ok(Self::new(home_path()?.join(Self::DEFAULT_FILE_NAME)))
    }

    fn read_key(path: &Path) -> std::result::Result<ed25519_dalek::SigningKey, StorageError> {
        let bytes = std::fs::read(path)?;
        let raw: [u8; 32] =
            bytes
                .try_into()
                .map_err(|v: Vec<u8>| StorageError::InvalidKeyLength {
                    expected: 32,
                    actual: v.len(),
                })?;
        Ok(ed25519_dalek::SigningKey::from_bytes(&raw))
    }
}

impl SigningKeyStorage for FileSigningKeyStorage {
    fn load_or_create(&self) -> std::result::Result<ed25519_dalek::SigningKey, StorageError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if self.path.exists() {
            return Self::read_key(&self.path);
        }

        let key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
        let raw_key = key.to_bytes();

        // Use create_new to prevent accidental overwrite if another process creates the key first.
        match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.path)
        {
            Ok(mut file) => {
                use std::io::Write as _;
                file.write_all(&raw_key)?;
                Ok(key)
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                Self::read_key(&self.path)
            }
            Err(err) => Err(StorageError::Io(err)),
        }
    }
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

    #[error("Wallet address is not configured")]
    WalletAddressNotConfigured,
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
    address: Option<Address>,
    chain_id: Option<ChainId>,
}

impl ArbiterSigner {
    pub async fn connect_grpc(url: ArbiterUrl) -> std::result::Result<Self, ConnectError> {
        let storage = FileSigningKeyStorage::from_default_location()?;
        Self::connect_grpc_with_storage(url, &storage).await
    }

    pub async fn connect_grpc_with_storage<S: SigningKeyStorage>(
        url: ArbiterUrl,
        storage: &S,
    ) -> std::result::Result<Self, ConnectError> {
        let key = storage.load_or_create()?;
        Self::connect_grpc_with_key(url, key).await
    }

    pub async fn connect_grpc_with_key(
        url: ArbiterUrl,
        key: ed25519_dalek::SigningKey,
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

        authenticate(&mut transport, &key).await?;

        Ok(Self {
            transport: Mutex::new(transport),
            address: None,
            chain_id: None,
        })
    }

    pub fn wallet_address(&self) -> Option<Address> {
        self.address
    }

    pub fn set_wallet_address(&mut self, address: Option<Address>) {
        self.address = address;
    }

    pub fn with_wallet_address(mut self, address: Address) -> Self {
        self.address = Some(address);
        self
    }

    pub fn with_chain_id(mut self, chain_id: ChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    fn build_sign_transaction_request(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> Result<ClientRequest> {
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

        let wallet_address = self
            .address
            .ok_or_else(|| Error::other(ClientSignError::WalletAddressNotConfigured))?;

        Ok(ClientRequest {
            payload: Some(ClientRequestPayload::EvmSignTransaction(
                EvmSignTransactionRequest {
                    wallet_address: wallet_address.as_slice().to_vec(),
                    rlp_transaction,
                },
            )),
        })
    }

    async fn execute_sign_transaction_request(&self, request: ClientRequest) -> Result<Signature> {
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

fn map_connect_error(code: i32) -> ConnectError {
    match client_connect_error::Code::try_from(code).unwrap_or(client_connect_error::Code::Unknown)
    {
        client_connect_error::Code::ApprovalDenied => ConnectError::ApprovalDenied,
        client_connect_error::Code::NoUserAgentsOnline => ConnectError::NoUserAgentsOnline,
        client_connect_error::Code::Unknown => ConnectError::UnexpectedAuthResponse,
    }
}

async fn send_auth_challenge_request(
    transport: &mut ClientTransport,
    key: &ed25519_dalek::SigningKey,
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
        .map_err(|_| ConnectError::UnexpectedAuthResponse)
}

async fn receive_auth_challenge(
    transport: &mut ClientTransport,
) -> std::result::Result<arbiter_proto::proto::client::AuthChallenge, ConnectError> {
    let response = transport
        .recv()
        .await
        .map_err(|_| ConnectError::MissingAuthChallenge)?;

    let payload = response.payload.ok_or(ConnectError::MissingAuthChallenge)?;
    match payload {
        ClientResponsePayload::AuthChallenge(challenge) => Ok(challenge),
        ClientResponsePayload::ClientConnectError(err) => Err(map_connect_error(err.code)),
        _ => Err(ConnectError::UnexpectedAuthResponse),
    }
}

async fn send_auth_challenge_solution(
    transport: &mut ClientTransport,
    key: &ed25519_dalek::SigningKey,
    challenge: arbiter_proto::proto::client::AuthChallenge,
) -> std::result::Result<(), ConnectError> {
    let challenge_payload = format_challenge(challenge.nonce, &challenge.pubkey);
    let signature = key.sign(&challenge_payload).to_bytes().to_vec();

    transport
        .send(ClientRequest {
            payload: Some(ClientRequestPayload::AuthChallengeSolution(
                AuthChallengeSolution { signature },
            )),
        })
        .await
        .map_err(|_| ConnectError::UnexpectedAuthResponse)
}

async fn receive_auth_confirmation(
    transport: &mut ClientTransport,
) -> std::result::Result<(), ConnectError> {
    let response = transport
        .recv()
        .await
        .map_err(|_| ConnectError::UnexpectedAuthResponse)?;

    let payload = response
        .payload
        .ok_or(ConnectError::UnexpectedAuthResponse)?;
    match payload {
        ClientResponsePayload::AuthOk(_) => Ok(()),
        ClientResponsePayload::ClientConnectError(err) => Err(map_connect_error(err.code)),
        _ => Err(ConnectError::UnexpectedAuthResponse),
    }
}

async fn authenticate(
    transport: &mut ClientTransport,
    key: &ed25519_dalek::SigningKey,
) -> std::result::Result<(), ConnectError> {
    send_auth_challenge_request(transport, key).await?;
    let challenge = receive_auth_challenge(transport).await?;
    send_auth_challenge_solution(transport, key, challenge).await?;
    receive_auth_confirmation(transport).await
}

#[async_trait]
impl Signer for ArbiterSigner {
    async fn sign_hash(&self, _hash: &B256) -> Result<Signature> {
        Err(Error::other(
            "hash-only signing is not supported for ArbiterSigner; use transaction signing",
        ))
    }

    fn address(&self) -> Address {
        self.address.unwrap_or(Address::ZERO)
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
        self.address.unwrap_or(Address::ZERO)
    }

    async fn sign_transaction(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> Result<Signature> {
        let request = self.build_sign_transaction_request(tx)?;
        self.execute_sign_transaction_request(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::{FileSigningKeyStorage, SigningKeyStorage, StorageError};

    fn unique_temp_key_path() -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "arbiter-client-key-{}-{}.bin",
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn file_storage_creates_and_reuses_key() {
        let path = unique_temp_key_path();
        let storage = FileSigningKeyStorage::new(path.clone());

        let key_a = storage
            .load_or_create()
            .expect("first load_or_create should create key");
        let key_b = storage
            .load_or_create()
            .expect("second load_or_create should read same key");

        assert_eq!(key_a.to_bytes(), key_b.to_bytes());
        assert!(path.exists());

        std::fs::remove_file(path).expect("temp key file should be removable");
    }

    #[test]
    fn file_storage_rejects_invalid_key_length() {
        let path = unique_temp_key_path();
        std::fs::write(&path, [42u8; 31]).expect("should write invalid key file");
        let storage = FileSigningKeyStorage::new(path.clone());

        let err = storage
            .load_or_create()
            .expect_err("storage should reject non-32-byte key file");

        match err {
            StorageError::InvalidKeyLength { expected, actual } => {
                assert_eq!(expected, 32);
                assert_eq!(actual, 31);
            }
            other => panic!("unexpected error: {other:?}"),
        }

        std::fs::remove_file(path).expect("temp key file should be removable");
    }
}
