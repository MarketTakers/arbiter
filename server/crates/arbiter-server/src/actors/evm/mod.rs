use alloy::{consensus::TxEip1559, primitives::Address, signers::Signature};
use diesel::{
    ExpressionMethods, OptionalExtension as _, QueryDsl, SelectableHelper as _, dsl::insert_into,
};
use diesel_async::RunQueryDsl;
use kameo::{Actor, actor::ActorRef, messages};
use rand::{SeedableRng, rng, rngs::StdRng};

use crate::{
    actors::keyholder::{CreateNew, Decrypt, KeyHolder},
    crypto::integrity::{self, Integrable, Verified, hashing::Hashable},
    db::{
        DatabaseError, DatabasePool,
        models::{self},
        schema,
    },
    evm::{
        self, ListError, RunKind,
        policies::{
            CombinedSettings, Grant, SharedGrantSettings, SpecificGrant, SpecificMeaning,
            ether_transfer::EtherTransfer, token_transfers::TokenTransfer,
        },
    },
    safe_cell::{SafeCell, SafeCellHandle as _},
};

pub use crate::evm::safe_signer;

/// Hashable structure for wallet integrity protection.
/// Binds the encrypted private key to the wallet address using HMAC.
pub struct EvmWalletIntegrity {
    pub address: Vec<u8>,       // 20-byte Ethereum address
    pub aead_encrypted_id: i32, // Reference to encrypted key material
}

impl Hashable for EvmWalletIntegrity {
    fn hash<H: sha2::Digest>(&self, hasher: &mut H) {
        hasher.update(&self.address);
        hasher.update(self.aead_encrypted_id.to_be_bytes());
    }
}

impl Integrable for EvmWalletIntegrity {
    const KIND: &'static str = "evm_wallet";
}

#[derive(Debug, thiserror::Error)]
pub enum SignTransactionError {
    #[error("Wallet not found")]
    WalletNotFound,

    #[error("Wallet integrity check failed")]
    WalletIntegrityCheckFailed,

    #[error(
        "Decrypted key does not correspond to wallet address (CRITICAL: possible key substitution attack)"
    )]
    KeyAddressMismatch,

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Keyholder error: {0}")]
    Keyholder(#[from] crate::actors::keyholder::Error),

    #[error("Keyholder mailbox error")]
    KeyholderSend,

    #[error("Signing error: {0}")]
    Signing(#[from] alloy::signers::Error),

    #[error("Policy error: {0}")]
    Vet(#[from] evm::VetError),

    #[error("Integrity error: {0}")]
    Integrity(#[from] integrity::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Keyholder error: {0}")]
    Keyholder(#[from] crate::actors::keyholder::Error),

    #[error("Keyholder mailbox error")]
    KeyholderSend,

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Integrity violation: {0}")]
    Integrity(#[from] integrity::Error),
}

#[derive(Actor)]
pub struct EvmActor {
    pub keyholder: ActorRef<KeyHolder>,
    pub db: DatabasePool,
    pub rng: StdRng,
    pub engine: evm::Engine,
}

impl EvmActor {
    pub fn new(keyholder: ActorRef<KeyHolder>, db: DatabasePool) -> Self {
        // is it safe to seed rng from system once?
        // todo: audit
        let rng = StdRng::from_rng(&mut rng());
        let engine = evm::Engine::new(db.clone(), keyholder.clone());
        Self {
            keyholder,
            db,
            rng,
            engine,
        }
    }
}

#[messages]
impl EvmActor {
    #[message]
    pub async fn generate(&mut self) -> Result<(Verified<i32>, Address), Error> {
        let (mut key_cell, address) = safe_signer::generate(&mut self.rng);

        let plaintext = key_cell.read_inline(|reader| SafeCell::new(reader.to_vec()));

        let aead_id: i32 = self
            .keyholder
            .ask(CreateNew { plaintext })
            .await
            .map_err(|_| Error::KeyholderSend)?;

        let mut conn = self.db.get().await.map_err(DatabaseError::from)?;
        let wallet_id = insert_into(schema::evm_wallet::table)
            .values(&models::NewEvmWallet {
                address: address.as_slice().to_vec(),
                aead_encrypted_id: aead_id,
            })
            .returning(schema::evm_wallet::id)
            .get_result(&mut conn)
            .await
            .map_err(DatabaseError::from)?;

        // Sign integrity envelope to bind encrypted key to wallet address
        let wallet_integrity = EvmWalletIntegrity {
            address: address.as_slice().to_vec(),
            aead_encrypted_id: aead_id,
        };
        let verified_wallet_id =
            integrity::sign_entity(&mut conn, &self.keyholder, &wallet_integrity, wallet_id)
                .await?;

        Ok((verified_wallet_id, address))
    }

    #[message]
    pub async fn list_wallets(&self) -> Result<Vec<(i32, Address)>, Error> {
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
    pub async fn useragent_create_grant(
        &mut self,
        basic: SharedGrantSettings,
        grant: SpecificGrant,
    ) -> Result<integrity::Verified<i32>, Error> {
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
    pub async fn useragent_delete_grant(&mut self, _grant_id: i32) -> Result<(), Error> {
        // let mut conn = self.db.get().await.map_err(DatabaseError::from)?;
        // let keyholder = self.keyholder.clone();

        // diesel_async::AsyncConnection::transaction(&mut conn, |conn| {
        //     Box::pin(async move {
        //         diesel::update(schema::evm_basic_grant::table)
        //             .filter(schema::evm_basic_grant::id.eq(grant_id))
        //             .set(schema::evm_basic_grant::revoked_at.eq(SqliteTimestamp::now()))
        //             .execute(conn)
        //             .await?;

        //         let signed = integrity::evm::load_signed_grant_by_basic_id(conn, grant_id).await?;

        //         diesel::result::QueryResult::Ok(())
        //     })
        // })
        // .await
        // .map_err(DatabaseError::from)?;

        // Ok(())
        todo!()
    }

    #[message]
    pub async fn useragent_list_grants(&mut self) -> Result<Vec<Grant<SpecificGrant>>, Error> {
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

        // Verify wallet integrity envelope
        let wallet = integrity::verify_entity(
            &mut conn,
            &self.keyholder,
            EvmWalletIntegrity {
                address: wallet.address.clone(),
                aead_encrypted_id: wallet.aead_encrypted_id,
            },
            wallet.id,
        )
        .await
        .map_err(|_| SignTransactionError::WalletIntegrityCheckFailed)?;

        let wallet_access = schema::evm_wallet_access::table
            .select(models::EvmWalletAccess::as_select())
            .filter(schema::evm_wallet_access::wallet_id.eq(wallet.entity_id))
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

        // Verify wallet integrity envelope to ensure encrypted key is bound to address
        let wallet = integrity::verify_entity(
            &mut conn,
            &self.keyholder,
            EvmWalletIntegrity {
                address: wallet.address.clone(),
                aead_encrypted_id: wallet.aead_encrypted_id,
            },
            wallet.id,
        )
        .await
        .map_err(|_| SignTransactionError::WalletIntegrityCheckFailed)?;

        let wallet_access = schema::evm_wallet_access::table
            .select(models::EvmWalletAccess::as_select())
            .filter(schema::evm_wallet_access::wallet_id.eq(wallet.entity_id))
            .filter(schema::evm_wallet_access::client_id.eq(client_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(DatabaseError::from)?
            .ok_or(SignTransactionError::WalletNotFound)?;
        drop(conn);

        let raw_key: SafeCell<Vec<u8>> = self
            .keyholder
            .ask(Decrypt {
                aead_id: wallet.aead_encrypted_id,
            })
            .await
            .map_err(|_| SignTransactionError::KeyholderSend)?;

        let signer = safe_signer::SafeSigner::from_cell(raw_key)?;

        // Verify that the decrypted key's derived address matches the wallet address
        // This prevents an attacker from substituting one wallet's key with another's even if they compromised the DB
        if signer.address() != wallet_address {
            return Err(SignTransactionError::KeyAddressMismatch);
        }

        self.engine
            .evaluate_transaction(wallet_access, transaction.clone(), RunKind::Execution)
            .await?;

        use alloy::network::TxSignerSync as _;
        Ok(signer.sign_transaction_sync(&mut transaction)?)
    }
}
