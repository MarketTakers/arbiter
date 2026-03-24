use arbiter_proto::{proto::arbiter_service_client::ArbiterServiceClient, url::ArbiterUrl};
use std::sync::Arc;
use terrors::{Broaden as _, OneOf};
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::ClientTlsConfig;

use crate::{
    auth::authenticate,
    errors::ConnectError,
    storage::{FileSigningKeyStorage, SigningKeyStorage},
    transport::{BUFFER_LENGTH, ClientTransport},
};

#[cfg(feature = "evm")]
use crate::errors::{ClientConnectionClosedError, ClientError};

#[cfg(feature = "evm")]
use crate::wallets::evm::ArbiterEvmWallet;

pub struct ArbiterClient {
    #[allow(dead_code)]
    transport: Arc<Mutex<ClientTransport>>,
}

impl ArbiterClient {
    pub async fn connect(url: ArbiterUrl) -> Result<Self, ConnectError> {
        let storage = FileSigningKeyStorage::from_default_location().broaden()?;
        Self::connect_with_storage(url, &storage).await
    }

    pub async fn connect_with_storage<S: SigningKeyStorage>(
        url: ArbiterUrl,
        storage: &S,
    ) -> Result<Self, ConnectError> {
        let key = storage.load_or_create().broaden()?;
        Self::connect_with_key(url, key).await
    }

    pub async fn connect_with_key(
        url: ArbiterUrl,
        key: ed25519_dalek::SigningKey,
    ) -> Result<Self, ConnectError> {
        let anchor = webpki::anchor_from_trusted_cert(&url.ca_cert)
            .map_err(OneOf::new)?
            .to_owned();
        let tls = ClientTlsConfig::new().trust_anchor(anchor);

        let channel = tonic::transport::Channel::from_shared(format!("{}:{}", url.host, url.port))
            .map_err(OneOf::new)?
            .tls_config(tls)
            .map_err(OneOf::new)?
            .connect()
            .await
            .map_err(OneOf::new)?;

        let mut client = ArbiterServiceClient::new(channel);
        let (tx, rx) = mpsc::channel(BUFFER_LENGTH);
        let response_stream = client
            .client(ReceiverStream::new(rx))
            .await
            .map_err(OneOf::new)?
            .into_inner();

        let mut transport = ClientTransport {
            sender: tx,
            receiver: response_stream,
        };

        authenticate(&mut transport, &key).await?;

        Ok(Self {
            transport: Arc::new(Mutex::new(transport)),
        })
    }

    #[cfg(feature = "evm")]
    pub async fn evm_wallets(&self) -> Result<Vec<ArbiterEvmWallet>, ClientError> {
        let _ = &self.transport;
        Err(OneOf::new(ClientConnectionClosedError))
    }
}
