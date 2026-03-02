use alloy::primitives::Address;
use diesel::{QueryDsl, SelectableHelper as _, dsl::insert_into};
use diesel_async::RunQueryDsl;
use kameo::{Actor, actor::ActorRef, messages};
use memsafe::MemSafe;
use rand::{SeedableRng, rng, rngs::StdRng};

use crate::{
    actors::keyholder::{CreateNew, KeyHolder},
    db::{self, DatabasePool, models, schema},
};

pub use crate::evm::safe_signer;

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
}

#[derive(Actor)]
pub struct EvmActor {
    pub keyholder: ActorRef<KeyHolder>,
    pub db: DatabasePool,
    pub rng: StdRng,
}

impl EvmActor {
    pub fn new(keyholder: ActorRef<KeyHolder>, db: DatabasePool) -> Self {
        // is it safe to seed rng from system once?
        // todo: audit
        let rng = StdRng::from_rng(&mut rng());
        Self { keyholder, db, rng }
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
