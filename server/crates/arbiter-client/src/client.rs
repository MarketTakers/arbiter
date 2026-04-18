#[cfg(feature = "evm")]
use crate::wallets::evm::ArbiterEvmWallet;
use crate::{
    StorageError,
    auth::{AuthError, authenticate},
    storage::{FileSigningKeyStorage, SigningKeyStorage},
    transport::{BUFFER_LENGTH, ClientTransport},
};
use arbiter_crypto::authn::SigningKey;
use arbiter_proto::{
    ClientMetadata, proto::arbiter_service_client::ArbiterServiceClient, url::ArbiterUrl,
};

use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::ClientTlsConfig;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("gRPC error")]
    Grpc(#[from] tonic::Status),

    #[error("Could not establish connection")]
    Connection(#[from] tonic::transport::Error),

    #[error("Invalid server URI")]
    InvalidUri(#[from] http::uri::InvalidUri),

    #[error("Invalid CA certificate")]
    InvalidCaCert(#[from] webpki::Error),

    #[error("Authentication error")]
    Authentication(#[from] AuthError),

    #[error("Storage error")]
    Storage(#[from] StorageError),
}

pub struct ArbiterClient {
    #[allow(dead_code)]
    transport: Arc<Mutex<ClientTransport>>,
}

impl ArbiterClient {
    pub async fn connect(url: ArbiterUrl, metadata: ClientMetadata) -> Result<Self, Error> {
        let storage = FileSigningKeyStorage::from_default_location()?;
        Self::connect_with_storage(url, metadata, &storage).await
    }

    pub async fn connect_with_storage<S: SigningKeyStorage>(
        url: ArbiterUrl,
        metadata: ClientMetadata,
        storage: &S,
    ) -> Result<Self, Error> {
        let key = storage.load_or_create()?;
        Self::connect_with_key(url, metadata, key).await
    }

    pub async fn connect_with_key(
        url: ArbiterUrl,
        metadata: ClientMetadata,
        key: SigningKey,
    ) -> Result<Self, Error> {
        let anchor = webpki::anchor_from_trusted_cert(&url.ca_cert)?.to_owned();
        let tls = ClientTlsConfig::new().trust_anchor(anchor);

        let channel =
            tonic::transport::Channel::from_shared(format!("https://{}:{}", url.host, url.port))?
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

        authenticate(&mut transport, metadata, &key).await?;

        Ok(Self {
            transport: Arc::new(Mutex::new(transport)),
        })
    }

    #[cfg(feature = "evm")]
    pub async fn evm_wallets(&self) -> Result<Vec<ArbiterEvmWallet>, Error> {
        todo!("fetch EVM wallet list from server")
    }
}
