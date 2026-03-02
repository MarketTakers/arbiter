pub mod safe_signer;

use alloy::{
    consensus::TxEip1559,
    primitives::TxKind,
};
use diesel::insert_into;
use diesel_async::{AsyncConnection, RunQueryDsl};

use crate::{
    db::{
        self,
        models::{EvmBasicGrant, NewEvmBasicGrant, SqliteTimestamp}, schema,
    },
    evm::policies::{
        EvalContext, FullGrant, Policy, SpecificMeaning, ether_transfer::EtherTransfer
    },
};

pub mod policies;
mod utils;

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

// Supporting only EIP-1559 transactions for now, but we can easily extend this to support legacy transactions if needed
pub struct Engine {
    db: db::DatabasePool,
}

impl Engine {
    pub fn new(db: db::DatabasePool) -> Self {
        Self { db }
    }

    pub async fn create_grant<P: Policy>(
        &self,
        client_id: i32,
        full_grant: FullGrant<P::Grant>,
    ) -> Result<i32, CreationError> {
        let mut conn = self.db.get().await?;

        let id = conn.transaction(|conn| {
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

    async fn perform_transaction<P: Policy>(&self, _context: EvalContext, _meaning: &P::Meaning) {
    }

    pub async fn analyze_transaction(
        &self,
        wallet_id: i32,
        client_id: i32,
        transaction: TxEip1559,
    ) -> Result<SpecificMeaning, AnalyzeError> {
        let TxKind::Call(to) = transaction.to else {
            return Err(AnalyzeError::ContractCreationNotSupported);
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
            return Ok(SpecificMeaning::EtherTransfer(meaning));
        }
        Err(AnalyzeError::UnsupportedTransactionType)
    }
}
