use alloy::{consensus::TxEip1559, primitives::Address, signers::Signature};
use diesel::{
    BoolExpressionMethods as _, ExpressionMethods, OptionalExtension as _, QueryDsl,
    SelectableHelper as _, dsl::insert_into,
};
use diesel_async::{AsyncConnection as _, RunQueryDsl};
use kameo::{Actor, actor::ActorRef, messages};
use rand::{SeedableRng, rng, rngs::StdRng};

use crate::{
    actors::keyholder::{CreateNew, Decrypt, KeyHolder},
    crypto::integrity,
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

#[derive(Debug, thiserror::Error)]
pub enum SignTransactionError {
    #[error("Wallet not found")]
    WalletNotFound,

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
    pub async fn generate(&mut self) -> Result<(i32, Address), Error> {
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

        Ok((wallet_id, address))
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
    pub async fn useragent_delete_grant(&mut self, grant_id: i32) -> Result<(), Error> {
        let mut conn = self.db.get().await.map_err(DatabaseError::from)?;

        // We intentionally perform a hard delete here to avoid leaving revoked grants and their
        // related rows as long-lived DB garbage. We also don't rely on SQLite FK cascades because
        // they can be disabled per-connection.
        conn.transaction(|conn| {
            Box::pin(async move {
                // First, resolve policy-specific rows by basic grant id.
                let token_grant_id: Option<i32> = schema::evm_token_transfer_grant::table
                    .select(schema::evm_token_transfer_grant::id)
                    .filter(schema::evm_token_transfer_grant::basic_grant_id.eq(grant_id))
                    .first::<i32>(conn)
                    .await
                    .optional()?;

                let ether_grant: Option<(i32, i32)> = schema::evm_ether_transfer_grant::table
                    .select((
                        schema::evm_ether_transfer_grant::id,
                        schema::evm_ether_transfer_grant::limit_id,
                    ))
                    .filter(schema::evm_ether_transfer_grant::basic_grant_id.eq(grant_id))
                    .first::<(i32, i32)>(conn)
                    .await
                    .optional()?;

                // Token-transfer: logs must be deleted before transaction logs (FK restrict).
                if let Some(token_grant_id) = token_grant_id {
                    diesel::delete(
                        schema::evm_token_transfer_log::table
                            .filter(schema::evm_token_transfer_log::grant_id.eq(token_grant_id)),
                    )
                    .execute(conn)
                    .await?;

                    diesel::delete(schema::evm_token_transfer_volume_limit::table.filter(
                        schema::evm_token_transfer_volume_limit::grant_id.eq(token_grant_id),
                    ))
                    .execute(conn)
                    .await?;

                    diesel::delete(
                        schema::evm_token_transfer_grant::table
                            .filter(schema::evm_token_transfer_grant::id.eq(token_grant_id)),
                    )
                    .execute(conn)
                    .await?;
                }

                // Shared transaction logs for any grant kind.
                diesel::delete(
                    schema::evm_transaction_log::table
                        .filter(schema::evm_transaction_log::grant_id.eq(grant_id)),
                )
                .execute(conn)
                .await?;

                // Ether-transfer: delete targets, grant row, then its limit row.
                if let Some((ether_grant_id, limit_id)) = ether_grant {
                    diesel::delete(schema::evm_ether_transfer_grant_target::table.filter(
                        schema::evm_ether_transfer_grant_target::grant_id.eq(ether_grant_id),
                    ))
                    .execute(conn)
                    .await?;

                    diesel::delete(
                        schema::evm_ether_transfer_grant::table
                            .filter(schema::evm_ether_transfer_grant::id.eq(ether_grant_id)),
                    )
                    .execute(conn)
                    .await?;

                    diesel::delete(
                        schema::evm_ether_transfer_limit::table
                            .filter(schema::evm_ether_transfer_limit::id.eq(limit_id)),
                    )
                    .execute(conn)
                    .await?;
                }

                // Integrity envelopes are not FK-constrained; delete only grant-related kinds to
                // avoid accidentally deleting other entities that share the same integer ID.
                let entity_id = grant_id.to_be_bytes().to_vec();
                diesel::delete(
                    schema::integrity_envelope::table
                        .filter(schema::integrity_envelope::entity_id.eq(entity_id))
                        .filter(
                            schema::integrity_envelope::entity_kind
                                .eq("EtherTransfer")
                                .or(schema::integrity_envelope::entity_kind.eq("TokenTransfer")),
                        ),
                )
                .execute(conn)
                .await?;

                // Finally remove the basic grant row itself (idempotent if it doesn't exist).
                diesel::delete(
                    schema::evm_basic_grant::table.filter(schema::evm_basic_grant::id.eq(grant_id)),
                )
                .execute(conn)
                .await?;

                diesel::result::QueryResult::Ok(())
            })
        })
        .await
        .map_err(DatabaseError::from)?;

        Ok(())
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

        let raw_key: SafeCell<Vec<u8>> = self
            .keyholder
            .ask(Decrypt {
                aead_id: wallet.aead_encrypted_id,
            })
            .await
            .map_err(|_| SignTransactionError::KeyholderSend)?;

        let signer = safe_signer::SafeSigner::from_cell(raw_key)?;

        self.engine
            .evaluate_transaction(wallet_access, transaction.clone(), RunKind::Execution)
            .await?;

        use alloy::network::TxSignerSync as _;
        Ok(signer.sign_transaction_sync(&mut transaction)?)
    }
}

#[cfg(test)]
mod tests;
