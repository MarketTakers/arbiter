use alloy::{
    consensus::SignableTransaction,
    network::TxSigner,
    primitives::{Address, B256, ChainId, Signature},
    signers::{Error, Result, Signer},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::transport::ClientTransport;

pub struct ArbiterEvmWallet {
    transport: Arc<Mutex<ClientTransport>>,
    address: Address,
    chain_id: Option<ChainId>,
}

impl ArbiterEvmWallet {
    pub(crate) fn new(transport: Arc<Mutex<ClientTransport>>, address: Address) -> Self {
        Self {
            transport,
            address,
            chain_id: None,
        }
    }

    pub fn address(&self) -> Address {
        self.address
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
}

#[async_trait]
impl Signer for ArbiterEvmWallet {
    async fn sign_hash(&self, _hash: &B256) -> Result<Signature> {
        Err(Error::other(
            "hash-only signing is not supported for ArbiterEvmWallet; use transaction signing",
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
impl TxSigner<Signature> for ArbiterEvmWallet {
    fn address(&self) -> Address {
        self.address
    }

    async fn sign_transaction(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> Result<Signature> {
        let _transport = self.transport.lock().await;
        self.validate_chain_id(tx)?;

        Err(Error::other(
            "transaction signing is not supported by current arbiter.client protocol",
        ))
    }
}
