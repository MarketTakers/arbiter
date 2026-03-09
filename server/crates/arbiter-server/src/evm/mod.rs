pub mod abi;
pub mod safe_signer;

use alloy::{consensus::TxEip1559, primitives::TxKind, signers::Signature};
use chrono::Utc;
use diesel::{QueryResult, insert_into};
use diesel_async::{AsyncConnection, RunQueryDsl};

use crate::{
    db::{
        self,
        models::{
            EvmBasicGrant, EvmTransactionLog, NewEvmBasicGrant, NewEvmTransactionLog,
            SqliteTimestamp,
        },
        schema::{self, evm_transaction_log},
    },
    evm::policies::{
        EvalContext, EvalViolation, FullGrant, Policy, SpecificMeaning,
        ether_transfer::EtherTransfer, token_transfers::TokenTransfer,
    },
};

pub mod policies;
mod utils;

/// Errors that can only occur once the transaction meaning is known (during policy evaluation)
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum PolicyError {
    #[error("Database connection pool error")]
    #[diagnostic(code(arbiter_server::evm::policy_error::pool))]
    Pool(#[from] db::PoolError),
    #[error("Database returned error")]
    #[diagnostic(code(arbiter_server::evm::policy_error::database))]
    Database(#[from] diesel::result::Error),
    #[error("Transaction violates policy: {0:?}")]
    #[diagnostic(code(arbiter_server::evm::policy_error::violation))]
    Violations(Vec<EvalViolation>),
    #[error("No matching grant found")]
    #[diagnostic(code(arbiter_server::evm::policy_error::no_matching_grant))]
    NoMatchingGrant,
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum VetError {
    #[error("Contract creation transactions are not supported")]
    #[diagnostic(code(arbiter_server::evm::vet_error::contract_creation_unsupported))]
    ContractCreationNotSupported,
    #[error("Engine can't classify this transaction")]
    #[diagnostic(code(arbiter_server::evm::vet_error::unsupported))]
    UnsupportedTransactionType,
    #[error("Policy evaluation failed: {1}")]
    #[diagnostic(code(arbiter_server::evm::vet_error::evaluated))]
    Evaluated(SpecificMeaning, #[source] PolicyError),
}


#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum SignError {
    #[error("Database connection pool error")]
    #[diagnostic(code(arbiter_server::evm::database_error))]
    Pool(#[from] db::PoolError),
    #[error("Database returned error")]
    #[diagnostic(code(arbiter_server::evm::database_error))]
    Database(#[from] diesel::result::Error),
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum AnalyzeError {
    #[error("Engine doesn't support granting permissions for contract creation")]
    #[diagnostic(code(arbiter_server::evm::analyze_error::contract_creation_not_supported))]
    ContractCreationNotSupported,

    #[error("Unsupported transaction type")]
    #[diagnostic(code(arbiter_server::evm::analyze_error::unsupported_transaction_type))]
    UnsupportedTransactionType,
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum CreationError {
    #[error("Database connection pool error")]
    #[diagnostic(code(arbiter_server::evm::creation_error::database_error))]
    Pool(#[from] db::PoolError),

    #[error("Database returned error")]
    #[diagnostic(code(arbiter_server::evm::creation_error::database_error))]
    Database(#[from] diesel::result::Error),
}

/// Controls whether a transaction should be executed or only validated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunKind {
    /// Validate and record the transaction
    Execution,
    /// Validate only, do not record
    CheckOnly,
}

// Supporting only EIP-1559 transactions for now, but we can easily extend this to support legacy transactions if needed
pub struct Engine {
    db: db::DatabasePool,
}

impl Engine {
    async fn vet_transaction<P: Policy>(
        &self,
        context: EvalContext,
        meaning: &P::Meaning,
        run_kind: RunKind,
    ) -> Result<(), PolicyError> {
        let mut conn = self.db.get().await?;

        let grant = P::try_find_grant(&context, &mut conn)
            .await?
            .ok_or(PolicyError::NoMatchingGrant)?;

        let violations = P::evaluate(&context, meaning, &grant, &mut conn).await?;
        if !violations.is_empty() {
            return Err(PolicyError::Violations(violations));
        } else if run_kind == RunKind::Execution {
            conn.transaction(|conn| {
                Box::pin(async move {
                    let log_id: i32 = insert_into(evm_transaction_log::table)
                        .values(&NewEvmTransactionLog {
                            grant_id: grant.shared_grant_id,
                            client_id: context.client_id,
                            wallet_id: context.wallet_id,
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
            .await?;
        }

        Ok(())
    }
}

impl Engine {
    pub fn new(db: db::DatabasePool) -> Self {
        Self { db }
    }

    pub async fn create_grant<P: Policy>(
        &self,
        client_id: i32,
        full_grant: FullGrant<P::Settings>,
    ) -> Result<i32, CreationError> {
        let mut conn = self.db.get().await?;

        let id = conn
            .transaction(|conn| {
                Box::pin(async move {
                    use schema::evm_basic_grant;

                    let basic_grant: EvmBasicGrant = insert_into(evm_basic_grant::table)
                        .values(&NewEvmBasicGrant {
                            wallet_id: full_grant.basic.wallet_id,
                            chain_id: full_grant.basic.chain as i32,
                            client_id: client_id,
                            valid_from: full_grant.basic.valid_from.map(SqliteTimestamp),
                            valid_until: full_grant.basic.valid_until.map(SqliteTimestamp),
                            max_gas_fee_per_gas: full_grant
                                .basic
                                .max_gas_fee_per_gas
                                .map(|fee| utils::u256_to_bytes(fee).to_vec()),
                            max_priority_fee_per_gas: full_grant
                                .basic
                                .max_priority_fee_per_gas
                                .map(|fee| utils::u256_to_bytes(fee).to_vec()),
                            rate_limit_count: full_grant
                                .basic
                                .rate_limit
                                .as_ref()
                                .map(|rl| rl.count as i32),
                            rate_limit_window_secs: full_grant
                                .basic
                                .rate_limit
                                .as_ref()
                                .map(|rl| rl.window.num_seconds() as i32),
                            revoked_at: None,
                        })
                        .returning(evm_basic_grant::all_columns)
                        .get_result(conn)
                        .await?;

                    P::create_grant(&basic_grant, &full_grant.specific, conn).await
                })
            })
            .await?;

        Ok(id)
    }

    pub async fn evaluate_transaction(
        &self,
        wallet_id: i32,
        client_id: i32,
        transaction: TxEip1559,
        run_kind: RunKind,
    ) -> Result<SpecificMeaning, VetError> {
        let TxKind::Call(to) = transaction.to else {
            return Err(VetError::ContractCreationNotSupported);
        };
        let context = policies::EvalContext {
            wallet_id,
            client_id,
            chain: transaction.chain_id,
            to: to,
            value: transaction.value,
            calldata: transaction.input.clone(),
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
