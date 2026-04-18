use crate::{
    actors::vault::Vault,
    crypto::integrity,
    db::{
        self, DatabaseError,
        models::{
            EvmBasicGrant, EvmWalletAccess, NewEvmBasicGrant, NewEvmTransactionLog, SqliteTimestamp,
        },
        schema::{self, evm_transaction_log},
    },
    evm::policies::{
        CombinedSettings, DatabaseID, EvalContext, EvalViolation, Grant, Policy,
        SharedGrantSettings, SpecificGrant, SpecificMeaning, ether_transfer::EtherTransfer,
        token_transfers::TokenTransfer,
    },
};

use alloy::{
    consensus::TxEip1559,
    primitives::{TxKind, U256},
};
use chrono::Utc;
use diesel::{ExpressionMethods as _, QueryDsl as _, QueryResult, insert_into, sqlite::Sqlite};
use diesel_async::{AsyncConnection, RunQueryDsl};
use kameo::actor::ActorRef;

pub mod abi;
pub mod safe_signer;

pub mod policies;
mod utils;

/// Errors that can only occur once the transaction meaning is known (during policy evaluation)
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("Database error")]
    Database(#[from] DatabaseError),
    #[error("Transaction violates policy: {0:?}")]
    Violations(Vec<EvalViolation>),
    #[error("No matching grant found")]
    NoMatchingGrant,

    #[error("Integrity error: {0}")]
    Integrity(#[from] integrity::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum VetError {
    #[error("Contract creation transactions are not supported")]
    ContractCreationNotSupported,
    #[error("Engine can't classify this transaction")]
    UnsupportedTransactionType,
    #[error("Policy evaluation failed: {1}")]
    Evaluated(SpecificMeaning, #[source] PolicyError),
}

#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error("Engine doesn't support granting permissions for contract creation")]
    ContractCreationNotSupported,

    #[error("Unsupported transaction type")]
    UnsupportedTransactionType,
}

#[derive(Debug, thiserror::Error)]
pub enum ListError {
    #[error("Database error")]
    Database(#[from] DatabaseError),

    #[error("Integrity verification failed for grant")]
    Integrity(#[from] integrity::Error),
}

/// Controls whether a transaction should be executed or only validated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunKind {
    /// Validate and record the transaction
    Execution,
    /// Validate only, do not record
    CheckOnly,
}

async fn check_shared_constraints(
    context: &EvalContext,
    shared: &SharedGrantSettings,
    shared_grant_id: DatabaseID,
    conn: &mut impl AsyncConnection<Backend = Sqlite>,
) -> QueryResult<Vec<EvalViolation>> {
    let mut violations = Vec::new();
    let now = Utc::now();

    if shared.chain != context.chain {
        violations.push(EvalViolation::MismatchingChainId {
            expected: shared.chain,
            actual: context.chain,
        });
        return Ok(violations);
    }

    // Validity window
    if shared.valid_from.is_some_and(|t| now < t) || shared.valid_until.is_some_and(|t| now > t) {
        violations.push(EvalViolation::InvalidTime);
    }

    // Gas fee caps
    let fee_exceeded = shared
        .max_gas_fee_per_gas
        .is_some_and(|cap| U256::from(context.max_fee_per_gas) > cap);
    let priority_exceeded = shared
        .max_priority_fee_per_gas
        .is_some_and(|cap| U256::from(context.max_priority_fee_per_gas) > cap);
    if fee_exceeded || priority_exceeded {
        violations.push(EvalViolation::GasLimitExceeded {
            max_gas_fee_per_gas: shared.max_gas_fee_per_gas,
            max_priority_fee_per_gas: shared.max_priority_fee_per_gas,
        });
    }

    // Transaction count rate limit
    if let Some(rate_limit) = &shared.rate_limit {
        let window_start = SqliteTimestamp(now - rate_limit.window);
        let count: i64 = evm_transaction_log::table
            .filter(evm_transaction_log::grant_id.eq(shared_grant_id))
            .filter(evm_transaction_log::signed_at.ge(window_start))
            .count()
            .get_result(conn)
            .await?;

        if count >= rate_limit.count.into() {
            violations.push(EvalViolation::RateLimitExceeded);
        }
    }

    Ok(violations)
}

// Supporting only EIP-1559 transactions for now, but we can easily extend this to support legacy transactions if needed
pub struct Engine {
    db: db::DatabasePool,
    vault: ActorRef<Vault>,
}

impl Engine {
    async fn vet_transaction<P: Policy>(
        &self,
        context: EvalContext,
        meaning: &P::Meaning,
        run_kind: RunKind,
    ) -> Result<(), PolicyError>
    where
        P::Settings: Clone,
    {
        let mut conn = self.db.get().await.map_err(DatabaseError::from)?;

        let grant = P::try_find_grant(&context, &mut conn)
            .await
            .map_err(DatabaseError::from)?
            .ok_or(PolicyError::NoMatchingGrant)?;

        integrity::verify_entity(&mut conn, &self.vault, &grant.settings, grant.id).await?;

        let mut violations = check_shared_constraints(
            &context,
            &grant.settings.shared,
            grant.common_settings_id,
            &mut conn,
        )
        .await
        .map_err(DatabaseError::from)?;
        violations.extend(
            P::evaluate(&context, meaning, &grant, &mut conn)
                .await
                .map_err(DatabaseError::from)?,
        );

        if !violations.is_empty() {
            return Err(PolicyError::Violations(violations));
        }

        if run_kind == RunKind::Execution {
            conn.transaction(|conn| {
                Box::pin(async move {
                    let log_id: i32 = insert_into(evm_transaction_log::table)
                        .values(&NewEvmTransactionLog {
                            grant_id: grant.common_settings_id,
                            wallet_access_id: context.target.id,
                            chain_id: context.chain.into(),
                            eth_value: utils::u256_to_bytes(context.value).to_vec(),
                            signed_at: Utc::now().into(),
                        })
                        .returning(evm_transaction_log::id)
                        .get_result(conn)
                        .await?;

                    P::record_transaction(&context, meaning, log_id, &grant, conn).await?;

                    QueryResult::Ok(())
                })
            })
            .await
            .map_err(DatabaseError::from)?;
        }

        Ok(())
    }
}

impl Engine {
    pub const fn new(db: db::DatabasePool, vault: ActorRef<Vault>) -> Self {
        Self { db, vault }
    }

    pub async fn create_grant<P: Policy>(
        &self,
        full_grant: CombinedSettings<P::Settings>,
    ) -> Result<i32, DatabaseError>
    where
        P::Settings: Clone,
    {
        let mut conn = self.db.get().await?;
        let vault = self.vault.clone();

        let id = conn
            .transaction(|conn| {
                Box::pin(async move {
                    use schema::evm_basic_grant;

                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::cast_possible_wrap,
                        clippy::as_conversions,
                        reason = "fixme! #86"
                    )]
                    let basic_grant: EvmBasicGrant = insert_into(evm_basic_grant::table)
                        .values(&NewEvmBasicGrant {
                            chain_id: full_grant.shared.chain.into(),
                            wallet_access_id: full_grant.shared.wallet_access_id,
                            valid_from: full_grant.shared.valid_from.map(SqliteTimestamp),
                            valid_until: full_grant.shared.valid_until.map(SqliteTimestamp),
                            max_gas_fee_per_gas: full_grant
                                .shared
                                .max_gas_fee_per_gas
                                .map(|fee| utils::u256_to_bytes(fee).to_vec()),
                            max_priority_fee_per_gas: full_grant
                                .shared
                                .max_priority_fee_per_gas
                                .map(|fee| utils::u256_to_bytes(fee).to_vec()),
                            rate_limit_count: full_grant
                                .shared
                                .rate_limit
                                .as_ref()
                                .map(|rl| rl.count as i32),
                            rate_limit_window_secs: full_grant
                                .shared
                                .rate_limit
                                .as_ref()
                                .map(|rl| rl.window.num_seconds() as i32),
                            revoked_at: None,
                        })
                        .returning(evm_basic_grant::all_columns)
                        .get_result(conn)
                        .await?;

                    P::create_grant(&basic_grant, &full_grant.specific, conn).await?;

                    integrity::sign_entity(conn, &vault, &full_grant, basic_grant.id)
                        .await
                        .map_err(|_| diesel::result::Error::RollbackTransaction)?;

                    QueryResult::Ok(basic_grant.id)
                })
            })
            .await?;

        Ok(id)
    }

    async fn list_one_kind<Kind: Policy, Y>(
        &self,
        conn: &mut impl AsyncConnection<Backend = Sqlite>,
    ) -> Result<impl Iterator<Item = Grant<Y>>, ListError>
    where
        Y: From<Kind::Settings>,
    {
        let all_grants = Kind::find_all_grants(conn)
            .await
            .map_err(DatabaseError::from)?;

        // Verify integrity of all grants before returning any results
        for grant in &all_grants {
            integrity::verify_entity(conn, &self.vault, &grant.settings, grant.id).await?;
        }

        Ok(all_grants.into_iter().map(|g| Grant {
            id: g.id,
            common_settings_id: g.common_settings_id,
            settings: g.settings.generalize(),
        }))
    }

    pub async fn list_all_grants(&self) -> Result<Vec<Grant<SpecificGrant>>, ListError> {
        let mut conn = self.db.get().await.map_err(DatabaseError::from)?;

        let mut grants: Vec<Grant<SpecificGrant>> = Vec::new();

        grants.extend(self.list_one_kind::<EtherTransfer, _>(&mut conn).await?);
        grants.extend(self.list_one_kind::<TokenTransfer, _>(&mut conn).await?);

        Ok(grants)
    }

    pub async fn evaluate_transaction(
        &self,
        target: EvmWalletAccess,
        transaction: TxEip1559,
        run_kind: RunKind,
    ) -> Result<SpecificMeaning, VetError> {
        let TxKind::Call(to) = transaction.to else {
            return Err(VetError::ContractCreationNotSupported);
        };
        let context = EvalContext {
            target,
            chain: transaction.chain_id,
            to,
            value: transaction.value,
            calldata: transaction.input.clone(),
            max_fee_per_gas: transaction.max_fee_per_gas,
            max_priority_fee_per_gas: transaction.max_priority_fee_per_gas,
        };

        if let Some(meaning) = EtherTransfer::analyze(&context) {
            return match self
                .vet_transaction::<EtherTransfer>(context, &meaning, run_kind)
                .await
            {
                Ok(()) => Ok(meaning.into()),
                Err(e) => Err(VetError::Evaluated(meaning.into(), e)),
            };
        }
        if let Some(meaning) = TokenTransfer::analyze(&context) {
            return match self
                .vet_transaction::<TokenTransfer>(context, &meaning, run_kind)
                .await
            {
                Ok(()) => Ok(meaning.into()),
                Err(e) => Err(VetError::Evaluated(meaning.into(), e)),
            };
        }

        Err(VetError::UnsupportedTransactionType)
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{Address, Bytes, U256, address};
    use chrono::{Duration, Utc};
    use diesel::{SelectableHelper, insert_into};
    use diesel_async::RunQueryDsl;
    use rstest::rstest;

    use crate::db::{
        self, DatabaseConnection,
        models::{
            EvmBasicGrant, EvmWalletAccess, NewEvmBasicGrant, NewEvmTransactionLog, SqliteTimestamp,
        },
        schema::{evm_basic_grant, evm_transaction_log},
    };
    use crate::evm::policies::{
        EvalContext, EvalViolation, SharedGrantSettings, TransactionRateLimit,
    };

    use super::check_shared_constraints;

    const WALLET_ACCESS_ID: i32 = 1;
    const CHAIN_ID: u64 = 1;
    const RECIPIENT: Address = address!("1111111111111111111111111111111111111111");

    fn context() -> EvalContext {
        EvalContext {
            target: EvmWalletAccess {
                id: WALLET_ACCESS_ID,
                wallet_id: 10,
                client_id: 20,
                created_at: SqliteTimestamp(Utc::now()),
            },
            chain: CHAIN_ID,
            to: RECIPIENT,
            value: U256::ZERO,
            calldata: Bytes::new(),
            max_fee_per_gas: 100,
            max_priority_fee_per_gas: 10,
        }
    }

    fn shared_settings() -> SharedGrantSettings {
        SharedGrantSettings {
            wallet_access_id: WALLET_ACCESS_ID,
            chain: CHAIN_ID,
            valid_from: None,
            valid_until: None,
            max_gas_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            rate_limit: None,
        }
    }

    async fn insert_basic_grant(
        conn: &mut DatabaseConnection,
        shared: &SharedGrantSettings,
    ) -> EvmBasicGrant {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            clippy::as_conversions,
            reason = "fixme! #86"
        )]
        insert_into(evm_basic_grant::table)
            .values(NewEvmBasicGrant {
                wallet_access_id: shared.wallet_access_id,
                chain_id: shared.chain.into(),
                valid_from: shared.valid_from.map(SqliteTimestamp),
                valid_until: shared.valid_until.map(SqliteTimestamp),
                max_gas_fee_per_gas: shared
                    .max_gas_fee_per_gas
                    .map(|fee| super::utils::u256_to_bytes(fee).to_vec()),
                max_priority_fee_per_gas: shared
                    .max_priority_fee_per_gas
                    .map(|fee| super::utils::u256_to_bytes(fee).to_vec()),
                rate_limit_count: shared.rate_limit.as_ref().map(|limit| limit.count as i32),
                rate_limit_window_secs: shared
                    .rate_limit
                    .as_ref()
                    .map(|limit| limit.window.num_seconds() as i32),
                revoked_at: None,
            })
            .returning(EvmBasicGrant::as_select())
            .get_result(conn)
            .await
            .unwrap()
    }

    #[rstest]
    #[case::matching_chain(CHAIN_ID, false)]
    #[case::mismatching_chain(CHAIN_ID + 1, true)]
    #[tokio::test]
    async fn check_shared_constraints_enforces_chain_id(
        #[case] context_chain: u64,
        #[case] expect_mismatch: bool,
    ) {
        let db = db::create_test_pool().await;
        let mut conn = db.get().await.unwrap();

        let context = EvalContext {
            chain: context_chain,
            ..context()
        };

        let violations = check_shared_constraints(&context, &shared_settings(), 999, &mut *conn)
            .await
            .unwrap();

        assert_eq!(
            violations
                .iter()
                .any(|violation| matches!(violation, EvalViolation::MismatchingChainId { .. })),
            expect_mismatch
        );

        if expect_mismatch {
            assert_eq!(violations.len(), 1);
        } else {
            assert!(violations.is_empty());
        }
    }

    #[rstest]
    #[case::valid_from_in_bounds(Some(Utc::now() - Duration::hours(1)), None, false)]
    #[case::valid_from_out_of_bounds(Some(Utc::now() + Duration::hours(1)), None, true)]
    #[case::valid_until_in_bounds(None, Some(Utc::now() + Duration::hours(1)), false)]
    #[case::valid_until_out_of_bounds(None, Some(Utc::now() - Duration::hours(1)), true)]
    #[tokio::test]
    async fn check_shared_constraints_enforces_validity_window(
        #[case] valid_from: Option<chrono::DateTime<Utc>>,
        #[case] valid_until: Option<chrono::DateTime<Utc>>,
        #[case] expect_invalid_time: bool,
    ) {
        let db = db::create_test_pool().await;
        let mut conn = db.get().await.unwrap();

        let shared = SharedGrantSettings {
            valid_from,
            valid_until,
            ..shared_settings()
        };

        let violations = check_shared_constraints(&context(), &shared, 999, &mut *conn)
            .await
            .unwrap();

        assert_eq!(
            violations
                .iter()
                .any(|violation| matches!(violation, EvalViolation::InvalidTime)),
            expect_invalid_time
        );

        if expect_invalid_time {
            assert_eq!(violations.len(), 1);
        } else {
            assert!(violations.is_empty());
        }
    }

    #[rstest]
    #[case::max_fee_within_limit(Some(U256::from(100u64)), None, 100, 10, false)]
    #[case::max_fee_exceeded(Some(U256::from(99u64)), None, 100, 10, true)]
    #[case::priority_fee_within_limit(None, Some(U256::from(10u64)), 100, 10, false)]
    #[case::priority_fee_exceeded(None, Some(U256::from(9u64)), 100, 10, true)]
    #[tokio::test]
    async fn check_shared_constraints_enforces_gas_fee_caps(
        #[case] max_gas_fee_per_gas: Option<U256>,
        #[case] max_priority_fee_per_gas: Option<U256>,
        #[case] actual_max_fee_per_gas: u128,
        #[case] actual_max_priority_fee_per_gas: u128,
        #[case] expect_gas_limit_violation: bool,
    ) {
        let db = db::create_test_pool().await;
        let mut conn = db.get().await.unwrap();

        let context = EvalContext {
            max_fee_per_gas: actual_max_fee_per_gas,
            max_priority_fee_per_gas: actual_max_priority_fee_per_gas,
            ..context()
        };

        let shared = SharedGrantSettings {
            max_gas_fee_per_gas,
            max_priority_fee_per_gas,
            ..shared_settings()
        };
        let violations = check_shared_constraints(&context, &shared, 999, &mut *conn)
            .await
            .unwrap();

        assert_eq!(
            violations
                .iter()
                .any(|violation| matches!(violation, EvalViolation::GasLimitExceeded { .. })),
            expect_gas_limit_violation
        );

        if expect_gas_limit_violation {
            assert_eq!(violations.len(), 1);
        } else {
            assert!(violations.is_empty());
        }
    }

    #[rstest]
    #[case::under_rate_limit(2, false)]
    #[case::at_rate_limit(1, true)]
    #[tokio::test]
    async fn check_shared_constraints_enforces_rate_limit(
        #[case] rate_limit_count: u32,
        #[case] expect_rate_limit_violation: bool,
    ) {
        let db = db::create_test_pool().await;
        let mut conn = db.get().await.unwrap();

        let shared = SharedGrantSettings {
            rate_limit: Some(TransactionRateLimit {
                count: rate_limit_count,
                window: Duration::hours(1),
            }),
            ..shared_settings()
        };

        let basic_grant = insert_basic_grant(&mut conn, &shared).await;

        insert_into(evm_transaction_log::table)
            .values(NewEvmTransactionLog {
                grant_id: basic_grant.id,
                wallet_access_id: WALLET_ACCESS_ID,
                chain_id: CHAIN_ID.into(),
                eth_value: super::utils::u256_to_bytes(U256::ZERO).to_vec(),
                signed_at: SqliteTimestamp(Utc::now()),
            })
            .execute(&mut *conn)
            .await
            .unwrap();

        let violations = check_shared_constraints(&context(), &shared, basic_grant.id, &mut *conn)
            .await
            .unwrap();

        assert_eq!(
            violations
                .iter()
                .any(|violation| matches!(violation, EvalViolation::RateLimitExceeded)),
            expect_rate_limit_violation
        );

        if expect_rate_limit_violation {
            assert_eq!(violations.len(), 1);
        } else {
            assert!(violations.is_empty());
        }
    }
}
