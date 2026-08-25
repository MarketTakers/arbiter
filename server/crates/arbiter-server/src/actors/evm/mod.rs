use crate::{
    actors::vault::{CreateNew, Decrypt, Vault},
    crypto::integrity::{self, Integrable},
    db::{
        DatabaseError, DatabasePool,
        models::{self, EvmWalletId},
        schema,
    },
    evm::{
        self, ListError, RunKind,
        policies::{
            CombinedSettings, Grant, SharedGrantSettings, SpecificGrant, SpecificMeaning,
            ether_transfer::EtherTransfer, token_transfers::TokenTransfer,
        },
    },
};
use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};

use alloy::{
    consensus::TxEip1559, network::TxSignerSync as _, primitives::Address, signers::Signature,
};
use diesel::{
    ExpressionMethods, OptionalExtension as _, QueryDsl, SelectableHelper as _, dsl::insert_into,
};
use diesel_async::RunQueryDsl;
use kameo::{Actor, actor::ActorRef, messages};
use rand::{SeedableRng, rng, rngs::StdRng};
use tracing::error;

pub use crate::evm::safe_signer;

/// Integrity guard that binds a wallet's encrypted key ID to its Ethereum address.
/// Both fields are included in the HMAC — swapping `aead_encrypted_id` in the DB
/// invalidates the envelope MAC, and the AEAD ciphertext is also bound to `address`
/// as AAD, so decryption fails too.
#[derive(arbiter_macros::Hashable)]
struct EvmWalletIntegrity {
    aead_encrypted_id: i32,
    address: Address,
}

impl Integrable for EvmWalletIntegrity {
    const KIND: &'static str = "evm_wallet";
}

#[derive(Debug, thiserror::Error)]
pub enum SignTransactionError {
    #[error("Wallet not found")]
    WalletNotFound,

    #[error("Decrypted key does not match requested wallet address")]
    KeyAddressMismatch,

    #[error("Internal signing error")]
    Internal,

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Vault error: {0}")]
    Vault(#[from] crate::actors::vault::Error),

    #[error("Vault mailbox error")]
    VaultSend,

    #[error("Signing error: {0}")]
    Signing(#[from] alloy::signers::Error),

    #[error("Policy error: {0}")]
    Vet(#[from] evm::VetError),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Vault error: {0}")]
    Vault(#[from] crate::actors::vault::Error),

    #[error("Vault mailbox error")]
    VaultSend,

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Integrity violation: {0}")]
    Integrity(#[from] integrity::Error),
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Self::Database(DatabaseError::from(e))
    }
}

#[derive(Actor)]
pub struct EvmActor {
    pub vault: ActorRef<Vault>,
    pub db: DatabasePool,
    pub rng: StdRng,
    pub engine: evm::Engine,
}

impl EvmActor {
    pub fn new(vault: ActorRef<Vault>, db: DatabasePool) -> Self {
        // is it safe to seed rng from system once?
        // todo: audit
        let rng = StdRng::from_rng(&mut rng());
        let engine = evm::Engine::new(db.clone(), vault.clone());
        Self {
            vault,
            db,
            rng,
            engine,
        }
    }
}

#[messages]
impl EvmActor {
    #[message]
    pub async fn generate(&mut self) -> Result<(i32, Address), Error> {
        let (mut key_cell, address) = safe_signer::generate(&mut self.rng);

        let plaintext = key_cell.read_inline(|reader| SafeCell::new(reader.to_vec()));

        let aead_id: i32 = self
            .vault
            .ask(CreateNew {
                plaintext,
                aad: address.as_slice().to_vec(),
            })
            .await
            .map_err(|_| Error::VaultSend)?;

        let mut conn = self.db.get().await.map_err(DatabaseError::from)?;
        let wallet_id = conn
            .exclusive_transaction(async |conn| {
                let wallet_id: i32 = insert_into(schema::evm_wallet::table)
                    .values(&models::NewEvmWallet {
                        address: address.as_slice().to_vec(),
                        aead_encrypted_id: aead_id,
                    })
                    .returning(schema::evm_wallet::id)
                    .get_result(conn)
                    .await
                    .map_err(DatabaseError::from)
                    .map_err(Error::Database)?;

                integrity::sign_entity(
                    conn,
                    &self.vault,
                    &EvmWalletIntegrity { address, aead_encrypted_id: aead_id },
                    wallet_id,
                )
                .await
                .map_err(Error::Integrity)?;

                Ok::<i32, Error>(wallet_id)
            })
            .await?;

        Ok((wallet_id, address))
    }

    #[message]
    pub async fn list_wallets(&self) -> Result<Vec<(EvmWalletId, Address)>, Error> {
        let mut conn = self.db.get().await.map_err(DatabaseError::from)?;
        let rows: Vec<models::EvmWallet> = schema::evm_wallet::table
            .select(models::EvmWallet::as_select())
            .load(&mut conn)
            .await
            .map_err(DatabaseError::from)?;

        Ok(rows
            .into_iter()
            .map(|w| (w.id, Address::from_slice(&w.address)))
            .collect())
    }
}

#[messages]
impl EvmActor {
    #[message]
    pub async fn operator_create_grant(
        &mut self,
        basic: SharedGrantSettings,
        grant: SpecificGrant,
    ) -> Result<i32, Error> {
        match grant {
            SpecificGrant::EtherTransfer(settings) => self
                .engine
                .create_grant::<EtherTransfer>(CombinedSettings {
                    shared: basic,
                    specific: settings,
                })
                .await
                .map_err(Error::from),
            SpecificGrant::TokenTransfer(settings) => self
                .engine
                .create_grant::<TokenTransfer>(CombinedSettings {
                    shared: basic,
                    specific: settings,
                })
                .await
                .map_err(Error::from),
        }
    }

    #[message]
    pub async fn useragent_delete_grant(
        &mut self,
        grant_id: i32,
    ) -> Result<(), Error> {
        self.engine
            .revoke_grant(grant_id)
            .await
            .map_err(Error::from)
    }

    #[message]
    pub async fn operator_list_grants(&mut self) -> Result<Vec<Grant<SpecificGrant>>, Error> {
        match self.engine.list_all_grants().await {
            Ok(grants) => Ok(grants),
            Err(ListError::Database(db_err)) => Err(Error::Database(db_err)),
            Err(ListError::Integrity(integrity_err)) => Err(Error::Integrity(integrity_err)),
        }
    }

    #[message]
    pub async fn shared_analyze_transaction(
        &mut self,
        client_id: i32,
        wallet_address: Address,
        transaction: TxEip1559,
    ) -> Result<SpecificMeaning, SignTransactionError> {
        let mut conn = self.db.get().await.map_err(DatabaseError::from)?;
        let wallet = schema::evm_wallet::table
            .select(models::EvmWallet::as_select())
            .filter(schema::evm_wallet::address.eq(wallet_address.as_slice()))
            .first(&mut conn)
            .await
            .optional()
            .map_err(DatabaseError::from)?
            .ok_or(SignTransactionError::WalletNotFound)?;
        let wallet_access = schema::evm_wallet_access::table
            .select(models::EvmWalletAccess::as_select())
            .filter(schema::evm_wallet_access::wallet_id.eq(wallet.id))
            .filter(schema::evm_wallet_access::client_id.eq(client_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(DatabaseError::from)?
            .ok_or(SignTransactionError::WalletNotFound)?;
        drop(conn);

        let meaning = self
            .engine
            .evaluate_transaction(wallet_access, transaction.clone(), RunKind::Execution)
            .await?;

        Ok(meaning)
    }

    #[message]
    pub async fn client_sign_transaction(
        &mut self,
        client_id: i32,
        wallet_address: Address,
        mut transaction: TxEip1559,
    ) -> Result<Signature, SignTransactionError> {
        let mut conn = self.db.get().await.map_err(DatabaseError::from)?;
        let wallet = schema::evm_wallet::table
            .select(models::EvmWallet::as_select())
            .filter(schema::evm_wallet::address.eq(wallet_address.as_slice()))
            .first(&mut conn)
            .await
            .optional()
            .map_err(DatabaseError::from)?
            .ok_or(SignTransactionError::WalletNotFound)?;
        let wallet_access = schema::evm_wallet_access::table
            .select(models::EvmWalletAccess::as_select())
            .filter(schema::evm_wallet_access::wallet_id.eq(wallet.id))
            .filter(schema::evm_wallet_access::client_id.eq(client_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(DatabaseError::from)?
            .ok_or(SignTransactionError::WalletNotFound)?;
        drop(conn);

        let mut conn = self.db.get().await.map_err(DatabaseError::from)?;
        let attestation = integrity::verify_entity(
            &mut conn,
            &self.vault,
            &EvmWalletIntegrity {
                address: wallet_address,
                aead_encrypted_id: wallet.aead_encrypted_id,
            },
            wallet.id.to_raw(),
        )
        .await
        .map_err(|e| {
            error!(?e, ?wallet.id, "EVM wallet integrity check failed");
            SignTransactionError::Internal
        })?;
        drop(conn);

        if attestation != integrity::AttestationStatus::Attested {
            error!(
                ?wallet.id,
                "EVM wallet integrity unavailable; refusing to sign"
            );
            return Err(SignTransactionError::Internal);
        }

        let raw_key: SafeCell<Vec<u8>> = self
            .vault
            .ask(Decrypt {
                aead_id: wallet.aead_encrypted_id,
                aad: wallet.address.clone(),
            })
            .await
            .map_err(|_| SignTransactionError::VaultSend)?;

        let signer = safe_signer::SafeSigner::from_cell(raw_key)?;

        if signer.address() != wallet_address {
            error!(
                expected = %wallet_address,
                actual = %signer.address(),
                "Decrypted private key address does not match requested wallet"
            );
            return Err(SignTransactionError::KeyAddressMismatch);
        }

        self.engine
            .evaluate_transaction(wallet_access, transaction.clone(), RunKind::Execution)
            .await?;

        Ok(signer.sign_transaction_sync(&mut transaction)?)
    }
}
