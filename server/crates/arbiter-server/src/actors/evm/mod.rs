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

    #[error(
        "Grant timestamp {0} is outside the i32 range a grant boundary column can store \
         (Unix seconds, so no later than 2038-01-19T03:14:07Z)"
    )]
    InvalidTimestamp(i64),

    #[error("Wallet access {0} is revoked or does not exist")]
    AccessNotActive(i32),
}

/// Converts a grant boundary from Unix seconds. `None` in means "unbounded"; a value the
/// boundary column cannot store is an error, never a silently different window.
///
/// The range is `i32`, not `i64`, because that is what actually reaches the database:
/// `SqliteTimestamp::to_sql` narrows to `i32` (`fixme! #84`), so `3_000_000_000` -- a
/// `valid_from` in 2065 -- would wrap to 1902 and open the grant immediately instead of in
/// forty years. Accepting only what round-trips keeps the grant that gets written the grant
/// that was voted on.
fn grant_timestamp(secs: Option<i64>) -> Result<Option<chrono::DateTime<chrono::Utc>>, Error> {
    secs.map(|s| {
        let storable = i32::try_from(s).map_err(|_| Error::InvalidTimestamp(s))?;
        chrono::DateTime::from_timestamp(i64::from(storable), 0).ok_or(Error::InvalidTimestamp(s))
    })
    .transpose()
}

/// Refuses an access id that is revoked or absent, so nothing hangs a grant off it.
async fn ensure_access_active(
    conn: &mut crate::db::DatabaseConnection,
    access_id: i32,
) -> Result<(), Error> {
    let active: bool = diesel::select(diesel::dsl::exists(
        schema::evm_wallet_access::table
            .filter(schema::evm_wallet_access::id.eq(access_id))
            .filter(schema::evm_wallet_access::revoked_at.is_null()),
    ))
    .get_result(conn)
    .await
    .map_err(DatabaseError::from)?;

    if active {
        Ok(())
    } else {
        Err(Error::AccessNotActive(access_id))
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
            .filter(schema::evm_wallet_access::revoked_at.is_null())
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
            .filter(schema::evm_wallet_access::revoked_at.is_null())
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

        // Revives a previously revoked row instead of conflicting on it forever:
        // `uniq_wallet_access` is a unique index on `(wallet_id, client_id)`. Visibility is
        // all this restores -- revocation closes the grants that hung off the access, so a
        // persistent grant needs its own vote again (§3.2). See
        // `peers::operator::session::handlers::revoke_wallet_access`.
        insert_into(schema::evm_wallet_access::table)
            .values((
                schema::evm_wallet_access::wallet_id.eq(EvmWalletId::from_raw(settings.wallet_id)),
                schema::evm_wallet_access::client_id.eq(settings.client_id),
            ))
            .on_conflict((
                schema::evm_wallet_access::wallet_id,
                schema::evm_wallet_access::client_id,
            ))
            .do_update()
            .set(schema::evm_wallet_access::revoked_at.eq(None::<models::SqliteTimestamp>))
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

        // A persistent grant is only as good as the visibility it hangs off (§3.2, two
        // separate votes). The proposal names the access id when it is created and can be
        // approved much later, so the access may have been revoked in between; a grant
        // against a revoked access would sit dormant and go live the moment anyone re-grants.
        let mut conn = self.db.get().await.map_err(DatabaseError::from)?;
        ensure_access_active(&mut conn, grant.wallet_access_id).await?;
        drop(conn);

        let volume = |limit: persistent_grant::VolumeLimit| VolumeRateLimit {
            max_volume: U256::from_be_bytes(limit.max_volume),
            window: Duration::seconds(limit.window_secs),
        };

        let basic = SharedGrantSettings {
            wallet_access_id: grant.wallet_access_id,
            chain: grant.chain_id,
            valid_from: grant_timestamp(grant.valid_from_secs)?,
            valid_until: grant_timestamp(grant.valid_until_secs)?,
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

#[cfg(test)]
mod tests {
    use super::{Error, EvmActor, ensure_access_active, grant_timestamp};
    use crate::db::{self, models, schema};

    use diesel::{ExpressionMethods as _, QueryDsl as _, dsl::insert_into};
    use diesel_async::RunQueryDsl;

    #[test]
    fn absent_timestamp_stays_absent() {
        assert!(grant_timestamp(None).unwrap().is_none());
    }

    #[test]
    fn in_range_timestamp_is_converted() {
        let converted = grant_timestamp(Some(1_800_000_000)).unwrap();
        assert_eq!(converted.unwrap().timestamp(), 1_800_000_000);
    }

    /// An unrepresentable expiry must not silently become "no expiry": that would widen the
    /// grant beyond what was voted on.
    #[test]
    fn out_of_range_timestamp_is_an_error() {
        let err = grant_timestamp(Some(i64::MAX)).unwrap_err();
        assert!(matches!(err, Error::InvalidTimestamp(i64::MAX)));
    }

    /// A `valid_from` past 2038 is representable as a `DateTime` but not as the `i32` the
    /// boundary column stores: `3_000_000_000` (2065) wraps to a negative, which reads back as
    /// 1902 and makes the grant active immediately. Refusing it is the only way the grant
    /// that lands can match the window that was voted on.
    #[test]
    fn a_timestamp_past_2038_is_an_error() {
        let past_2038 = 3_000_000_000_i64;
        assert!(
            chrono::DateTime::from_timestamp(past_2038, 0).is_some(),
            "the fixture must be a date chrono accepts, or it proves nothing about storage"
        );

        let err = grant_timestamp(Some(past_2038)).unwrap_err();
        assert!(
            matches!(err, Error::InvalidTimestamp(got) if got == past_2038),
            "expected an out-of-range error, got {err:?}"
        );
    }

    /// The last second the boundary column can hold must still be accepted: the range check
    /// has to stop at what storage can take, not short of it.
    #[test]
    fn the_last_storable_timestamp_is_accepted() {
        let converted = grant_timestamp(Some(i64::from(i32::MAX))).unwrap();
        assert_eq!(converted.unwrap().timestamp(), i64::from(i32::MAX));
    }

    /// Seeds a wallet, a client and one access row between them, and returns the access id.
    async fn seed_access(conn: &mut db::DatabaseConnection) -> i32 {
        let root_key_id: models::RootKeyHistoryId = insert_into(schema::root_key_history::table)
            .values(&models::NewRootKeyHistory {
                ciphertext: vec![0u8; 32],
                tag: vec![0u8; 16],
                root_key_encryption_nonce: vec![0u8; 24],
                data_encryption_nonce: vec![0u8; 24],
                schema_version: 1,
                salt: vec![0u8; 16],
            })
            .returning(schema::root_key_history::id)
            .get_result(conn)
            .await
            .unwrap();

        let aead_id: i32 = insert_into(schema::aead_encrypted::table)
            .values(&models::NewAeadEncrypted {
                ciphertext: vec![0u8; 32],
                tag: vec![0u8; 16],
                current_nonce: vec![0u8; 24],
                schema_version: 1,
                associated_root_key_id: root_key_id,
                created_at: chrono::Utc::now().into(),
            })
            .returning(schema::aead_encrypted::id)
            .get_result(conn)
            .await
            .unwrap();

        let wallet_id: models::EvmWalletId = insert_into(schema::evm_wallet::table)
            .values((
                schema::evm_wallet::address.eq(rand::random::<[u8; 20]>().to_vec()),
                schema::evm_wallet::aead_encrypted_id.eq(aead_id),
            ))
            .returning(schema::evm_wallet::id)
            .get_result(conn)
            .await
            .unwrap();

        let metadata_id: i32 = insert_into(schema::client_metadata::table)
            .values(schema::client_metadata::name.eq("test"))
            .returning(schema::client_metadata::id)
            .get_result(conn)
            .await
            .unwrap();

        let client_id: i32 = insert_into(schema::program_client::table)
            .values((
                schema::program_client::public_key.eq(rand::random::<[u8; 32]>().to_vec()),
                schema::program_client::metadata_id.eq(metadata_id),
            ))
            .returning(schema::program_client::id)
            .get_result(conn)
            .await
            .unwrap();

        insert_into(schema::evm_wallet_access::table)
            .values((
                schema::evm_wallet_access::wallet_id.eq(wallet_id),
                schema::evm_wallet_access::client_id.eq(client_id),
            ))
            .returning(schema::evm_wallet_access::id)
            .get_result(conn)
            .await
            .unwrap()
    }

    /// Both directions, so a guard that refused everything could not pass: a live access is
    /// let through, a revoked one is not.
    #[tokio::test]
    async fn only_a_live_access_passes_the_grant_guard() {
        let pool = db::create_test_pool().await;
        let mut conn = pool.get().await.unwrap();

        let access_id = seed_access(&mut conn).await;
        ensure_access_active(&mut conn, access_id)
            .await
            .expect("a live access must pass");

        diesel::update(schema::evm_wallet_access::table)
            .filter(schema::evm_wallet_access::id.eq(access_id))
            .set(schema::evm_wallet_access::revoked_at.eq(models::SqliteTimestamp::now()))
            .execute(&mut conn)
            .await
            .unwrap();

        let err = ensure_access_active(&mut conn, access_id)
            .await
            .expect_err("a revoked access must be refused");
        assert!(
            matches!(err, Error::AccessNotActive(got) if got == access_id),
            "expected AccessNotActive, got {err:?}"
        );
    }

    /// The guard has to be wired into the executor, not just exist: an approved persistent
    /// grant whose access was revoked between proposal and approval must not create a grant
    /// that would go live again the moment anyone re-grants that access (§3.2).
    #[tokio::test]
    async fn an_approved_persistent_grant_refuses_a_revoked_access() {
        use crate::actors::{GlobalActors, vault::Vault};
        use crate::db::proposal::persistent_grant;
        use kameo::actor::Spawn as _;

        let pool = db::create_test_pool().await;
        let mut conn = pool.get().await.unwrap();

        let access_id = seed_access(&mut conn).await;
        diesel::update(schema::evm_wallet_access::table)
            .filter(schema::evm_wallet_access::id.eq(access_id))
            .set(schema::evm_wallet_access::revoked_at.eq(models::SqliteTimestamp::now()))
            .execute(&mut conn)
            .await
            .unwrap();
        drop(conn);

        let vault = Vault::spawn(
            Vault::new(pool.clone(), GlobalActors::spawn_message_bus())
                .await
                .unwrap(),
        );
        let mut evm_actor = EvmActor::new(vault, pool.clone());

        let err = evm_actor
            .create_persistent_grant(persistent_grant::Settings {
                wallet_access_id: access_id,
                chain_id: 1,
                valid_from_secs: None,
                valid_until_secs: None,
                max_gas_fee_per_gas: None,
                max_priority_fee_per_gas: None,
                rate_limit: None,
                specific: persistent_grant::Specific::EtherTransfer {
                    targets: vec![[0u8; 20]],
                    limit: persistent_grant::VolumeLimit {
                        max_volume: [0u8; 32],
                        window_secs: 3600,
                    },
                },
            })
            .await
            .expect_err("a grant against a revoked access must be refused");
        assert!(
            matches!(err, Error::AccessNotActive(got) if got == access_id),
            "expected AccessNotActive, got {err:?}"
        );

        let grants: i64 = schema::evm_basic_grant::table
            .filter(schema::evm_basic_grant::wallet_access_id.eq(access_id))
            .count()
            .get_result(&mut pool.get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(
            grants, 0,
            "no grant row may be written for a revoked access"
        );
    }
}
