use crate::{
    actors::{
        proposal_manager::events::ProposalApproved,
        vault::{CreateNew, Decrypt, Vault},
    },
    crypto::integrity,
    db::{
        DatabaseError, DatabasePool,
        models::{self, EvmWalletId, ProposalId},
        proposal::{ProposalKind, grant_wallet_access, one_off_transaction, persistent_grant},
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
use kameo::{Actor, actor::ActorRef, messages, prelude::Message};
use rand::{SeedableRng, rng, rngs::StdRng};
use tracing::error;

pub use crate::evm::safe_signer;

#[derive(Debug, thiserror::Error)]
pub enum SignTransactionError {
    #[error("Wallet not found")]
    WalletNotFound,

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

    #[error("Signing error: {0}")]
    Sign(#[from] SignTransactionError),
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
            .ask(CreateNew { plaintext })
            .await
            .map_err(|_| Error::VaultSend)?;

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
    pub async fn operator_delete_grant(&mut self, grant_id: i32) -> Result<(), Error> {
        let mut conn = self.db.get().await.map_err(DatabaseError::from)?;

        let affected = diesel::update(schema::evm_basic_grant::table)
            .filter(schema::evm_basic_grant::id.eq(grant_id))
            .set(schema::evm_basic_grant::revoked_at.eq(models::SqliteTimestamp::now()))
            .execute(&mut conn)
            .await
            .map_err(DatabaseError::from)?;

        if affected == 0 {
            return Err(Error::Database(DatabaseError::from(
                diesel::result::Error::NotFound,
            )));
        }

        Ok(())
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

        let raw_key: SafeCell<Vec<u8>> = self
            .vault
            .ask(Decrypt {
                aead_id: wallet.aead_encrypted_id,
            })
            .await
            .map_err(|_| SignTransactionError::VaultSend)?;

        let signer = safe_signer::SafeSigner::from_cell(raw_key)?;

        self.engine
            .evaluate_transaction(wallet_access, transaction.clone(), RunKind::Execution)
            .await?;

        Ok(signer.sign_transaction_sync(&mut transaction)?)
    }
}

impl Message<ProposalApproved> for EvmActor {
    type Reply = ();

    /// Every subscriber sees every approval and acts only on the kinds it owns.
    async fn handle(
        &mut self,
        msg: ProposalApproved,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let result = match msg.kind {
            ProposalKind::GrantWalletAccess(settings) => self.grant_wallet_access(&settings).await,
            ProposalKind::ApprovePersistentGrant(settings) => {
                self.create_persistent_grant(*settings).await
            }
            ProposalKind::ApproveOneOffTransaction(settings) => {
                self.sign_one_off_transaction(msg.id, *settings).await
            }
            _ => return,
        };

        if let Err(error) = result {
            error!(
                ?error,
                proposal_id = msg.id.to_raw(),
                "Failed to execute an approved proposal"
            );
        }
    }
}

impl EvmActor {
    async fn grant_wallet_access(
        &mut self,
        settings: &grant_wallet_access::Settings,
    ) -> Result<(), Error> {
        let mut conn = self.db.get().await.map_err(DatabaseError::from)?;

        insert_into(schema::evm_wallet_access::table)
            .values((
                schema::evm_wallet_access::wallet_id.eq(EvmWalletId::from_raw(settings.wallet_id)),
                schema::evm_wallet_access::client_id.eq(settings.client_id),
            ))
            .execute(&mut conn)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }

    async fn create_persistent_grant(
        &mut self,
        grant: persistent_grant::Settings,
    ) -> Result<(), Error> {
        use crate::evm::policies::{
            TransactionRateLimit, VolumeRateLimit, ether_transfer, token_transfers,
        };
        use alloy::primitives::U256;
        use chrono::Duration;

        let volume = |limit: persistent_grant::VolumeLimit| VolumeRateLimit {
            max_volume: U256::from_be_bytes(limit.max_volume),
            window: Duration::seconds(limit.window_secs),
        };

        let basic = SharedGrantSettings {
            wallet_access_id: grant.wallet_access_id,
            chain: grant.chain_id,
            valid_from: grant
                .valid_from_secs
                .and_then(|s| chrono::DateTime::from_timestamp(s, 0)),
            valid_until: grant
                .valid_until_secs
                .and_then(|s| chrono::DateTime::from_timestamp(s, 0)),
            max_gas_fee_per_gas: grant.max_gas_fee_per_gas.map(U256::from_be_bytes),
            max_priority_fee_per_gas: grant.max_priority_fee_per_gas.map(U256::from_be_bytes),
            rate_limit: grant.rate_limit.map(|r| TransactionRateLimit {
                count: r.count,
                window: Duration::seconds(r.window_secs),
            }),
        };

        let specific = match grant.specific {
            persistent_grant::Specific::EtherTransfer { targets, limit } => {
                SpecificGrant::EtherTransfer(ether_transfer::Settings {
                    target: targets.into_iter().map(Address::from).collect(),
                    limit: volume(limit),
                })
            }
            persistent_grant::Specific::TokenTransfer {
                token_contract,
                receiver,
                volume_limits,
            } => SpecificGrant::TokenTransfer(token_transfers::Settings {
                token_contract: Address::from(token_contract),
                target: receiver.map(Address::from),
                volume_limits: volume_limits.into_iter().map(volume).collect(),
            }),
        };

        self.operator_create_grant(basic, specific).await?;

        Ok(())
    }

    async fn sign_one_off_transaction(
        &mut self,
        proposal_id: ProposalId,
        tx: one_off_transaction::Settings,
    ) -> Result<(), Error> {
        use alloy::{
            eips::eip2930::AccessList,
            primitives::{Bytes, TxKind, U256},
        };

        let transaction = TxEip1559 {
            chain_id: tx.chain_id,
            nonce: tx.nonce,
            gas_limit: tx.gas_limit,
            max_fee_per_gas: tx.max_fee_per_gas,
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
            to: TxKind::Call(Address::from(tx.to)),
            value: U256::from_be_bytes(tx.value),
            input: Bytes::from(tx.input),
            access_list: AccessList::default(),
        };

        let signature = self
            .client_sign_transaction(tx.client_id, Address::from(tx.wallet_address), transaction)
            .await?;

        let mut conn = self.db.get().await.map_err(DatabaseError::from)?;
        one_off_transaction::store_signature(proposal_id, &signature, &mut conn)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }
}
