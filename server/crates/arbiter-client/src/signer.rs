use alloy::{
    consensus::SignableTransaction,
    network::TxSigner,
    primitives::{Address, B256, ChainId, Signature},
    signers::{Error, Result, Signer},
};
use arbiter_proto::{
    proto::arbiter_service_client::ArbiterServiceClient,
    url::ArbiterUrl,
};
use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::ClientTlsConfig;

use crate::{
    auth::{ConnectError, authenticate},
    storage::{FileSigningKeyStorage, SigningKeyStorage},
    transport::{BUFFER_LENGTH, ClientSignError, ClientTransport},
};

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
