use alloy::{
    consensus::SignableTransaction,
    network::TxSigner,
    primitives::{Address, B256, ChainId, Signature},
    signers::{Error, Result, Signer},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use arbiter_proto::proto::{
    client::{
        ClientRequest,
        client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
        evm::{
            self as proto_evm, request::Payload as EvmRequestPayload,
            response::Payload as EvmResponsePayload,
        },
    },
    evm::{
        EvmSignTransactionRequest,
        evm_sign_transaction_response::Result as EvmSignTransactionResult,
    },
    shared::evm::TransactionEvalError,
};

use crate::transport::{ClientTransport, next_request_id};

/// A typed error payload returned by [`ArbiterEvmWallet`] transaction signing.
///
/// This is wrapped into `alloy::signers::Error::Other`, so consumers can downcast by [`TryFrom`] and
/// interpret the concrete policy evaluation failure instead of parsing strings.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ArbiterEvmSignTransactionError {
    #[error("transaction rejected by policy: {0:?}")]
    PolicyEval(TransactionEvalError),
}

impl<'a> TryFrom<&'a Error> for &'a ArbiterEvmSignTransactionError {
    type Error = ();

    fn try_from(value: &'a Error) -> Result<Self, Self::Error> {
        if let Error::Other(inner) = value
            && let Some(eval_error) = inner.downcast_ref()
        {
            Ok(eval_error)
        } else {
            Err(())
        }
    }
}

pub struct ArbiterEvmWallet {
    transport: Arc<Mutex<ClientTransport>>,
    address: Address,
    chain_id: Option<ChainId>,
}

impl ArbiterEvmWallet {
    #[expect(
        dead_code,
        reason = "constructor may be used in future extensions, e.g. to support wallet listing"
    )]
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
        self.validate_chain_id(tx)?;

        let mut transport = self.transport.lock().await;
        let request_id = next_request_id();
        let rlp_transaction = tx.encoded_for_signing();

        transport
            .send(ClientRequest {
                request_id,
                payload: Some(ClientRequestPayload::Evm(proto_evm::Request {
                    payload: Some(EvmRequestPayload::SignTransaction(
                        EvmSignTransactionRequest {
                            wallet_address: self.address.to_vec(),
                            rlp_transaction,
                        },
                    )),
                })),
            })
            .await
            .map_err(|_| Error::other("failed to send evm sign transaction request"))?;

        let response = transport
            .recv()
            .await
            .map_err(|_| Error::other("failed to receive evm sign transaction response"))?;

        if response.request_id != Some(request_id) {
            return Err(Error::other(
                "received mismatched response id for evm sign transaction",
            ));
        }

        let payload = response
            .payload
            .ok_or_else(|| Error::other("missing evm sign transaction response payload"))?;

        let ClientResponsePayload::Evm(proto_evm::Response {
            payload: Some(payload),
        }) = payload
        else {
            return Err(Error::other(
                "unexpected response payload for evm sign transaction request",
            ));
        };

        let EvmResponsePayload::SignTransaction(response) = payload else {
            return Err(Error::other(
                "unexpected evm response payload for sign transaction request",
            ));
        };

        let result = response
            .result
            .ok_or_else(|| Error::other("missing evm sign transaction result"))?;

        match result {
            EvmSignTransactionResult::Signature(signature) => {
                Signature::try_from(signature.as_slice())
                    .map_err(|_| Error::other("invalid signature returned by server"))
            }
            EvmSignTransactionResult::EvalError(eval_error) => Err(Error::other(
                ArbiterEvmSignTransactionError::PolicyEval(eval_error),
            )),
            EvmSignTransactionResult::Error(code) => Err(Error::other(format!(
                "server failed to sign transaction with error code {code}"
            ))),
        }
    }
}
