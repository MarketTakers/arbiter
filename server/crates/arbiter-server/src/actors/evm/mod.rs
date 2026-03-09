use alloy::{consensus::TxEip1559, network::TxSigner, primitives::Address, signers::Signature};
use diesel::{ExpressionMethods, OptionalExtension as _, QueryDsl, SelectableHelper as _, dsl::insert_into};
use diesel_async::RunQueryDsl;
use kameo::{Actor, actor::ActorRef, messages};
use memsafe::MemSafe;
use rand::{SeedableRng, rng, rngs::StdRng};

use crate::{
    actors::keyholder::{CreateNew, Decrypt, KeyHolder},
    db::{self, DatabasePool, models::{self, EvmBasicGrant, SqliteTimestamp}, schema},
    evm::{
        self, RunKind,
        policies::{
            FullGrant, SharedGrantSettings, SpecificGrant, SpecificMeaning,
            ether_transfer::EtherTransfer,
            token_transfers::TokenTransfer,
        },
    },
};

pub use crate::evm::safe_signer;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum SignTransactionError {
    #[error("Wallet not found")]
    #[diagnostic(code(arbiter::evm::sign::wallet_not_found))]
    WalletNotFound,

    #[error("Database error: {0}")]
    #[diagnostic(code(arbiter::evm::sign::database))]
    Database(#[from] diesel::result::Error),

    #[error("Database pool error: {0}")]
    #[diagnostic(code(arbiter::evm::sign::pool))]
    Pool(#[from] db::PoolError),

    #[error("Keyholder error: {0}")]
    #[diagnostic(code(arbiter::evm::sign::keyholder))]
    Keyholder(#[from] crate::actors::keyholder::Error),

    #[error("Keyholder mailbox error")]
    #[diagnostic(code(arbiter::evm::sign::keyholder_send))]
    KeyholderSend,

    #[error("Signing error: {0}")]
    #[diagnostic(code(arbiter::evm::sign::signing))]
    Signing(#[from] alloy::signers::Error),

    #[error("Policy error: {0}")]
    #[diagnostic(code(arbiter::evm::sign::vet))]
    Vet(#[from] evm::VetError),
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("Keyholder error: {0}")]
    #[diagnostic(code(arbiter::evm::keyholder))]
    Keyholder(#[from] crate::actors::keyholder::Error),

    #[error("Keyholder mailbox error")]
    #[diagnostic(code(arbiter::evm::keyholder_send))]
    KeyholderSend,

    #[error("Database error: {0}")]
    #[diagnostic(code(arbiter::evm::database))]
    Database(#[from] diesel::result::Error),

    #[error("Database pool error: {0}")]
    #[diagnostic(code(arbiter::evm::database_pool))]
    DatabasePool(#[from] db::PoolError),

    #[error("Grant creation error: {0}")]
    #[diagnostic(code(arbiter::evm::creation))]
    Creation(#[from] evm::CreationError),
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
        let engine = evm::Engine::new(db.clone());
        Self { keyholder, db, rng, engine }
    }
}

#[messages]
impl EvmActor {
    #[message]
    pub async fn generate(&mut self) -> Result<Address, Error> {
        let (mut key_cell, address) = safe_signer::generate(&mut self.rng);

        // Move raw key bytes into a Vec<u8> MemSafe for KeyHolder
        let plaintext = {
            let reader = key_cell.read().expect("MemSafe read");
            MemSafe::new(reader.to_vec()).expect("MemSafe allocation")
        };

        let aead_id: i32 = self
            .keyholder
            .ask(CreateNew { plaintext })
            .await
            .map_err(|_| Error::KeyholderSend)?;

        let mut conn = self.db.get().await?;
        insert_into(schema::evm_wallet::table)
            .values(&models::NewEvmWallet {
                address: address.as_slice().to_vec(),
                aead_encrypted_id: aead_id,
            })
            .execute(&mut conn)
            .await?;

        Ok(address)
    }

    #[message]
    pub async fn list_wallets(&self) -> Result<Vec<Address>, Error> {
        let mut conn = self.db.get().await?;
        let rows: Vec<models::EvmWallet> = schema::evm_wallet::table
            .select(models::EvmWallet::as_select())
            .load(&mut conn)
            .await?;

        Ok(rows
            .into_iter()
            .map(|w| Address::from_slice(&w.address))
            .collect())
    }
}

#[messages]
impl EvmActor {
    #[message]
    pub async fn useragent_create_grant(
        &mut self,
        client_id: i32,
        basic: SharedGrantSettings,
        grant: SpecificGrant,
    ) -> Result<i32, evm::CreationError> {
        match grant {
            SpecificGrant::EtherTransfer(settings) => {
                self.engine
                    .create_grant::<EtherTransfer>(client_id, FullGrant { basic, specific: settings })
                    .await
            }
            SpecificGrant::TokenTransfer(settings) => {
                self.engine
                    .create_grant::<TokenTransfer>(client_id, FullGrant { basic, specific: settings })
                    .await
            }
        }
    }

    #[message]
    pub async fn useragent_delete_grant(&mut self, grant_id: i32) -> Result<(), Error> {
        let mut conn = self.db.get().await?;
        diesel::update(schema::evm_basic_grant::table)
            .filter(schema::evm_basic_grant::id.eq(grant_id))
            .set(schema::evm_basic_grant::revoked_at.eq(SqliteTimestamp::now()))
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    #[message]
    pub async fn useragent_list_grants(
        &mut self,
        wallet_id: Option<i32>,
    ) -> Result<Vec<EvmBasicGrant>, Error> {
        let mut conn = self.db.get().await?;
        let mut query = schema::evm_basic_grant::table
            .select(EvmBasicGrant::as_select())
            .filter(schema::evm_basic_grant::revoked_at.is_null())
            .into_boxed();
        if let Some(wid) = wallet_id {
            query = query.filter(schema::evm_basic_grant::wallet_id.eq(wid));
        }
        Ok(query.load(&mut conn).await?)
    }

    #[message]
    pub async fn shared_analyze_transaction(
        &mut self,
        client_id: i32,
        wallet_address: Address,
        transaction: TxEip1559,
    ) -> Result<SpecificMeaning, SignTransactionError> {
        let mut conn = self.db.get().await?;
        let wallet = schema::evm_wallet::table
            .select(models::EvmWallet::as_select())
            .filter(schema::evm_wallet::address.eq(wallet_address.as_slice()))
            .first(&mut conn)
            .await
            .optional()?
            .ok_or(SignTransactionError::WalletNotFound)?;
        drop(conn);

        let meaning = self.engine
            .evaluate_transaction(wallet.id, client_id, transaction.clone(), RunKind::Execution)
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
        let mut conn = self.db.get().await?;
        let wallet = schema::evm_wallet::table
            .select(models::EvmWallet::as_select())
            .filter(schema::evm_wallet::address.eq(wallet_address.as_slice()))
            .first(&mut conn)
            .await
            .optional()?
            .ok_or(SignTransactionError::WalletNotFound)?;
        drop(conn);

        let raw_key: MemSafe<Vec<u8>> = self
            .keyholder
            .ask(Decrypt { aead_id: wallet.aead_encrypted_id })
            .await
            .map_err(|_| SignTransactionError::KeyholderSend)?;

        let signer = safe_signer::SafeSigner::from_memsafe(raw_key)?;

        self.engine
            .evaluate_transaction(wallet.id, client_id, transaction.clone(), RunKind::Execution)
            .await?;

        use alloy::network::TxSignerSync as _;
        Ok(signer.sign_transaction_sync(&mut transaction)?)
    }
}
