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
            AuthChallengeRequest, AuthChallengeSolution, AuthResult, ClientRequest, ClientResponse,
            client_request::Payload as ClientRequestPayload,
            client_response::Payload as ClientResponsePayload,
        },
    },
    url::ArbiterUrl,
};
use async_trait::async_trait;
use ed25519_dalek::Signer as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI32, Ordering};
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::ClientTlsConfig;

const BUFFER_LENGTH: usize = 16;
static NEXT_REQUEST_ID: AtomicI32 = AtomicI32::new(1);

fn next_request_id() -> i32 {
    NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

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

    fn validate_chain_id(&self, tx: &mut dyn SignableTransaction<Signature>) -> Result<()> {
        if let Some(chain_id) = self.chain_id
            && !tx.set_chain_id_checked(chain_id)
        {
            return Err(Error::TransactionChainIdMismatch {
                signer: chain_id,
                tx: tx.chain_id().unwrap(),
            });
        }

        Ok(())
    }

    fn ensure_wallet_address(&self) -> Result<Address> {
        let wallet_address = self
            .address
            .ok_or_else(|| Error::other(ClientSignError::WalletAddressNotConfigured))?;

        Ok(wallet_address)
    }
}

fn map_auth_result(code: i32) -> ConnectError {
    match AuthResult::try_from(code).unwrap_or(AuthResult::Unspecified) {
        AuthResult::ApprovalDenied => ConnectError::ApprovalDenied,
        AuthResult::NoUserAgentsOnline => ConnectError::NoUserAgentsOnline,
        AuthResult::Unspecified
        | AuthResult::Success
        | AuthResult::InvalidKey
        | AuthResult::InvalidSignature
        | AuthResult::Internal => ConnectError::UnexpectedAuthResponse,
    }
}

async fn send_auth_challenge_request(
    transport: &mut ClientTransport,
    key: &ed25519_dalek::SigningKey,
) -> std::result::Result<(), ConnectError> {
    transport
        .send(ClientRequest {
            request_id: next_request_id(),
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
        ClientResponsePayload::AuthResult(result) => Err(map_auth_result(result)),
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
            request_id: next_request_id(),
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
        ClientResponsePayload::AuthResult(result)
            if AuthResult::try_from(result).ok() == Some(AuthResult::Success) =>
        {
            Ok(())
        }
        ClientResponsePayload::AuthResult(result) => Err(map_auth_result(result)),
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
        let _transport = self.transport.lock().await;
        self.validate_chain_id(tx)?;
        let _ = self.ensure_wallet_address()?;

        Err(Error::other(
            "transaction signing is not supported by current arbiter.client protocol",
        ))
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
