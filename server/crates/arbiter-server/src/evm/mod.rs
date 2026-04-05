pub mod abi;
pub mod safe_signer;

use alloy::{
    consensus::TxEip1559,
    primitives::{TxKind, U256},
};
use chrono::Utc;
use diesel::{ExpressionMethods as _, QueryDsl as _, QueryResult, insert_into, sqlite::Sqlite};
use diesel_async::{AsyncConnection, RunQueryDsl};
use kameo::actor::ActorRef;

use crate::{
    actors::keyholder::KeyHolder,
    crypto::integrity,
    db::{
        self, DatabaseError,
        models::{
            EvmBasicGrant, EvmWalletAccess, NewEvmBasicGrant, NewEvmTransactionLog, SqliteTimestamp,
        },
        schema::{self, evm_transaction_log},
    },
    evm::policies::{
        DatabaseID, EvalContext, EvalViolation, Grant, Policy, CombinedSettings, SharedGrantSettings,
        SpecificGrant, SpecificMeaning, ether_transfer::EtherTransfer,
        token_transfers::TokenTransfer,
    },
};

pub mod policies;
mod utils;

/// Errors that can only occur once the transaction meaning is known (during policy evaluation)
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("Database error")]
    Database(#[from] crate::db::DatabaseError),
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
    Database(#[from] crate::db::DatabaseError),

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

        if count >= rate_limit.count as i64 {
            violations.push(EvalViolation::RateLimitExceeded);
        }
    }

    Ok(violations)
}

// Supporting only EIP-1559 transactions for now, but we can easily extend this to support legacy transactions if needed
pub struct Engine {
    db: db::DatabasePool,
    keyholder: ActorRef<KeyHolder>,
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

        integrity::verify_entity(&mut conn, &self.keyholder, &grant.settings, grant.id).await?;

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
                            chain_id: context.chain as i32,
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
    pub fn new(db: db::DatabasePool, keyholder: ActorRef<KeyHolder>) -> Self {
        Self { db, keyholder }
    }

    pub async fn create_grant<P: Policy>(
        &self,
        full_grant: CombinedSettings<P::Settings>,
    ) -> Result<i32, DatabaseError>
    where
        P::Settings: Clone,
    {
        let mut conn = self.db.get().await?;
        let keyholder = self.keyholder.clone();

        let id = conn
            .transaction(|conn| {
                Box::pin(async move {
                    use schema::evm_basic_grant;

                    let basic_grant: EvmBasicGrant = insert_into(evm_basic_grant::table)
                        .values(&NewEvmBasicGrant {
                            chain_id: full_grant.shared.chain as i32,
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

                    integrity::sign_entity(
                        conn,
                        &keyholder,
                        &full_grant,
                    basic_grant.id,
                    )
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
            integrity::verify_entity(conn, &self.keyholder, &grant.settings, grant.id).await?;
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
        let context = policies::EvalContext {
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
