use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};

use alloy::{
    consensus::SignableTransaction,
    network::{TxSigner, TxSignerSync},
    primitives::{Address, B256, ChainId, Signature},
    signers::{Error, Result, Signer, SignerSync, utils::secret_key_to_address},
};
use async_trait::async_trait;
use k256::ecdsa::{self, RecoveryId, SigningKey, signature::hazmat::PrehashSigner};
use std::sync::Mutex;

/// An Ethereum signer that stores its secp256k1 secret key inside a
/// hardware-protected [`MemSafe`] cell.
///
/// The underlying memory page is kept non-readable/non-writable at rest.
/// Access is temporarily elevated only for the duration of each signing
/// operation, then immediately revoked.
///
/// Because [`MemSafe::read`] requires `&mut self` while the [`Signer`] trait
/// requires `&self`, the cell is wrapped in a [`Mutex`].
pub struct SafeSigner {
    key: Mutex<SafeCell<SigningKey>>,
    address: Address,
    chain_id: Option<ChainId>,
}

impl std::fmt::Debug for SafeSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SafeSigner")
            .field("address", &self.address)
            .field("chain_id", &self.chain_id)
            .finish()
    }
}

/// Generates a secp256k1 secret key directly inside a [`MemSafe`] cell.
///
/// Random bytes are written in-place into protected memory, then validated
/// as a legal scalar on the secp256k1 curve (the scalar must be in
/// `[1, n)` where `n` is the curve order — roughly 1-in-2^128 chance of
/// rejection, but we retry to be correct).
///
/// Returns the protected key bytes and the derived Ethereum address.
pub fn generate(rng: &mut impl rand::Rng) -> (SafeCell<[u8; 32]>, Address) {
    loop {
        let mut cell = SafeCell::new_inline_default(|w: &mut [u8; 32]| {
            rng.fill_bytes(w);
        });

        let reader = cell.read();
        if let Ok(sk) = SigningKey::from_slice(reader.as_ref()) {
            let address = secret_key_to_address(&sk);
            drop(reader);
            return (cell, address);
        }
    }
}

impl SafeSigner {
    /// Reconstructs a `SafeSigner` from key material held in a [`MemSafe`] buffer.
    ///
    /// The key bytes are read from protected memory, parsed as a secp256k1
    /// scalar, and immediately moved into a new [`MemSafe`] cell. The raw
    /// bytes are never exposed outside this function.
    pub fn from_cell(mut cell: SafeCell<Vec<u8>>) -> Result<Self> {
        let reader = cell.read();
        let sk = SigningKey::from_slice(reader.as_slice()).map_err(Error::other)?;
        drop(reader);
        Self::new(sk)
    }

    /// Creates a new `SafeSigner` by moving the signing key into a protected
    /// memory region.
    pub fn new(key: SigningKey) -> Result<Self> {
        let address = secret_key_to_address(&key);
        let cell = SafeCell::new(key);
        Ok(Self {
            key: Mutex::new(cell),
            address,
            chain_id: None,
        })
    }

    #[expect(clippy::significant_drop_tightening, reason = "false positive")]
    fn sign_hash_inner(&self, hash: &B256) -> Result<Signature> {
        let mut cell = self.key.lock().expect("SafeSigner mutex poisoned");
        let reader = cell.read();
        let sig: (ecdsa::Signature, RecoveryId) = reader.sign_prehash(hash.as_ref())?;
        Ok(sig.into())
    }

    fn sign_tx_inner(&self, tx: &mut dyn SignableTransaction<Signature>) -> Result<Signature> {
        if let Some(chain_id) = self.chain_id
            && !tx.set_chain_id_checked(chain_id)
        {
            return Err(Error::TransactionChainIdMismatch {
                signer: chain_id,
                tx: tx.chain_id().expect("Chain ID is guaranteed to be set"),
            });
        }
        self.sign_hash_inner(&tx.signature_hash())
            .map_err(Error::other)
    }
}

#[async_trait]
impl Signer for SafeSigner {
    #[inline]
    async fn sign_hash(&self, hash: &B256) -> Result<Signature> {
        self.sign_hash_inner(hash)
    }

    #[inline]
    fn address(&self) -> Address {
        self.address
    }

    #[inline]
    fn chain_id(&self) -> Option<ChainId> {
        self.chain_id
    }

    #[inline]
    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        self.chain_id = chain_id;
    }
}

impl SignerSync for SafeSigner {
    #[inline]
    fn sign_hash_sync(&self, hash: &B256) -> Result<Signature> {
        self.sign_hash_inner(hash)
    }

    #[inline]
    fn chain_id_sync(&self) -> Option<ChainId> {
        self.chain_id
    }
}

#[async_trait]
impl TxSigner<Signature> for SafeSigner {
    fn address(&self) -> Address {
        self.address
    }

    async fn sign_transaction(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> Result<Signature> {
        self.sign_tx_inner(tx)
    }
}

impl TxSignerSync<Signature> for SafeSigner {
    fn address(&self) -> Address {
        self.address
    }

    fn sign_transaction_sync(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> Result<Signature> {
        self.sign_tx_inner(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::signers::local::PrivateKeySigner;

    #[test]
    fn sign_and_recover() {
        let pk = PrivateKeySigner::random();
        let key = pk.into_credential();
        let signer = SafeSigner::new(key).unwrap();
        let message = b"hello arbiter";
        let sig = signer.sign_message_sync(message).unwrap();
        let recovered = sig.recover_address_from_msg(message).unwrap();
        assert_eq!(recovered, Signer::address(&signer));
    }

    #[test]
    fn chain_id_roundtrip() {
        let pk = PrivateKeySigner::random();
        let key = pk.into_credential();
        let mut signer = SafeSigner::new(key).unwrap();
        assert_eq!(Signer::chain_id(&signer), None);
        signer.set_chain_id(Some(1337));
        assert_eq!(Signer::chain_id(&signer), Some(1337));
    }
}
