use arbiter_proto::{ClientMetadata, proto::arbiter_service_client::ArbiterServiceClient, url::ArbiterUrl};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::ClientTlsConfig;

use crate::{
    auth::{ConnectError, authenticate},
    storage::{FileSigningKeyStorage, SigningKeyStorage},
    transport::{BUFFER_LENGTH, ClientTransport},
};

#[cfg(feature = "evm")]
use crate::wallets::evm::ArbiterEvmWallet;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("gRPC error")]
    Grpc(#[from] tonic::Status),

    #[error("Connection closed by server")]
    ConnectionClosed,
}

pub struct ArbiterClient {
    #[allow(dead_code)]
    transport: Arc<Mutex<ClientTransport>>,
}

impl ArbiterClient {
    pub async fn connect(url: ArbiterUrl, metadata: ClientMetadata) -> Result<Self, ConnectError> {
        let storage = FileSigningKeyStorage::from_default_location()?;
        Self::connect_with_storage(url, metadata, &storage).await
    }

    pub async fn connect_with_storage<S: SigningKeyStorage>(
        url: ArbiterUrl,
        metadata: ClientMetadata,
        storage: &S,
    ) -> Result<Self, ConnectError> {
        let key = storage.load_or_create()?;
        Self::connect_with_key(url, metadata, key).await
    }

    pub async fn connect_with_key(
        url: ArbiterUrl,
        metadata: ClientMetadata,
        key: ed25519_dalek::SigningKey,
    ) -> Result<Self, ConnectError> {
        let anchor = webpki::anchor_from_trusted_cert(&url.ca_cert)?.to_owned();
        let tls = ClientTlsConfig::new().trust_anchor(anchor);

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

        authenticate(&mut transport, metadata, &key).await?;

        Ok(Self {
            transport: Arc::new(Mutex::new(transport)),
        })
    }

    #[cfg(feature = "evm")]
    pub async fn evm_wallets(&self) -> Result<Vec<ArbiterEvmWallet>, ClientError> {
        todo!("fetch EVM wallet list from server")
    }
}
