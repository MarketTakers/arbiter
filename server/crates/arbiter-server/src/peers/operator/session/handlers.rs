use super::{Error, OperatorSession};
use crate::{
    actors::{
        evm::{
            ClientSignTransaction, Generate, ListWallets, OperatorCreateGrant, OperatorDeleteGrant,
            OperatorListGrants, SignTransactionError as EvmSignError,
        },
        flow_coordinator::client_connect_approval::ClientApprovalAnswer,
        vault::VaultState,
    },
    db::models::{
        EvmWalletAccess, EvmWalletId, NewEvmWalletAccess, ProgramClient, ProgramClientMetadata,
    },
    evm::policies::{Grant, SpecificGrant},
};
use arbiter_crypto::authn;

use alloy::{consensus::TxEip1559, primitives::Address, signers::Signature};
use diesel::{ExpressionMethods as _, QueryDsl as _, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use kameo::{error::SendError, messages, prelude::Context};
use tracing::error;

#[derive(Debug, Error)]
pub enum SignTransactionError {
    #[error("Policy evaluation failed")]
    Vet(#[from] crate::evm::VetError),

    #[error("Internal signing error")]
    Internal,
}

#[derive(Debug, Error)]
pub enum GrantMutationError {
    #[error("Vault is sealed")]
    VaultSealed,

    #[error("Internal grant mutation error")]
    Internal,
}

#[messages]
impl OperatorSession {
    #[message]
    pub(crate) async fn handle_query_vault_state(&mut self) -> Result<VaultState, Error> {
        use crate::actors::vault::GetState;

        let vault_state = match self.props.actors.vault.ask(GetState {}).await {
            Ok(state) => state,
            Err(err) => {
                error!(?err, actor = "operator", "vault.query.failed");
                return Err(Error::internal("Vault is in broken state"));
            }
        };

        Ok(vault_state)
    }
}

#[messages]
impl OperatorSession {
    #[message]
    pub(crate) async fn handle_evm_wallet_create(&mut self) -> Result<(i32, Address), Error> {
        match self.props.actors.evm.ask(Generate {}).await {
            Ok(address) => Ok(address),
            Err(SendError::HandlerError(err)) => Err(Error::internal(format!(
                "EVM wallet generation failed: {err}"
            ))),
            Err(err) => {
                error!(?err, "EVM actor unreachable during wallet create");
                Err(Error::internal("EVM actor unreachable"))
            }
        }
    }

    #[message]
    pub(crate) async fn handle_evm_wallet_list(
        &mut self,
    ) -> Result<Vec<(EvmWalletId, Address)>, Error> {
        match self.props.actors.evm.ask(ListWallets {}).await {
            Ok(wallets) => Ok(wallets),
            Err(err) => {
                error!(?err, "EVM wallet list failed");
                Err(Error::internal("Failed to list EVM wallets"))
            }
        }
    }
}

#[messages]
impl OperatorSession {
    #[message]
    pub(crate) async fn handle_grant_list(&mut self) -> Result<Vec<Grant<SpecificGrant>>, Error> {
        match self.props.actors.evm.ask(OperatorListGrants {}).await {
            Ok(grants) => Ok(grants),
            Err(err) => {
                error!(?err, "EVM grant list failed");
                Err(Error::internal("Failed to list EVM grants"))
            }
        }
    }

    #[message]
    pub(crate) async fn handle_grant_create(
        &mut self,
        basic: crate::evm::policies::SharedGrantSettings,
        grant: SpecificGrant,
    ) -> Result<i32, GrantMutationError> {
        match self
            .props
            .actors
            .evm
            .ask(OperatorCreateGrant { basic, grant })
            .await
        {
            Ok(grant_id) => Ok(grant_id),
            Err(err) => {
                error!(?err, "EVM grant create failed");
                Err(GrantMutationError::Internal)
            }
        }
    }

    #[message]
    pub(crate) async fn handle_grant_delete(
        &mut self,
        grant_id: i32,
    ) -> Result<(), GrantMutationError> {
        match self
            .props
            .actors
            .evm
            .ask(OperatorDeleteGrant { grant_id })
            .await
        {
            Ok(()) => Ok(()),
            Err(err) => {
                error!(?err, "EVM grant delete failed");
                Err(GrantMutationError::Internal)
            }
        }
    }

    #[message]
    pub(crate) async fn handle_sign_transaction(
        &mut self,
        client_id: i32,
        wallet_address: Address,
        transaction: TxEip1559,
    ) -> Result<Signature, SignTransactionError> {
        match self
            .props
            .actors
            .evm
            .ask(ClientSignTransaction {
                client_id,
                wallet_address,
                transaction,
            })
            .await
        {
            Ok(signature) => Ok(signature),
            Err(SendError::HandlerError(EvmSignError::Vet(vet_error))) => {
                Err(SignTransactionError::Vet(vet_error))
            }
            Err(err) => {
                error!(?err, "EVM sign transaction failed in operator session");
                Err(SignTransactionError::Internal)
            }
        }
    }

    #[message]
    pub(crate) async fn handle_grant_evm_wallet_access(
        &mut self,
        entries: Vec<NewEvmWalletAccess>,
    ) -> Result<(), Error> {
        let mut conn = self.props.db.get().await?;
        conn.transaction(async |conn| {
            use crate::db::schema::evm_wallet_access;

            for entry in entries {
                diesel::insert_into(evm_wallet_access::table)
                    .values(&entry)
                    .on_conflict_do_nothing()
                    .execute(&mut *conn)
                    .await?;
            }

            Result::<_, Error>::Ok(())
        })
        .await?;
        Ok(())
    }

    #[message]
    pub(crate) async fn handle_revoke_evm_wallet_access(
        &mut self,
        entries: Vec<i32>,
    ) -> Result<(), Error> {
        let mut conn = self.props.db.get().await?;
        conn.transaction(async |conn| {
            use crate::db::schema::evm_wallet_access;
            for entry in entries {
                diesel::delete(evm_wallet_access::table)
                    .filter(evm_wallet_access::wallet_id.eq(entry))
                    .execute(&mut *conn)
                    .await?;
            }

            Result::<_, Error>::Ok(())
        })
        .await?;
        Ok(())
    }

    #[message]
    pub(crate) async fn handle_list_wallet_access(
        &mut self,
    ) -> Result<Vec<EvmWalletAccess>, Error> {
        use crate::db::schema::evm_wallet_access;
        let mut conn = self.props.db.get().await?;
        let access_entries = evm_wallet_access::table
            .select(EvmWalletAccess::as_select())
            .load::<_>(&mut conn)
            .await?;
        Ok(access_entries)
    }
}

#[messages]
impl OperatorSession {
    #[message(ctx)]
    pub(crate) async fn handle_new_client_approve(
        &mut self,
        approved: bool,
        pubkey: authn::PublicKey,
        ctx: &mut Context<Self, Result<(), Error>>,
    ) -> Result<(), Error> {
        let Some(pending_approval) = self.pending_client_approvals.remove(&pubkey.to_bytes())
        else {
            error!("Received client connection response for unknown client");
            return Err(Error::internal("Unknown client in connection response"));
        };

        pending_approval
            .controller
            .tell(ClientApprovalAnswer { approved })
            .await
            .map_err(|err| {
                error!(
                    ?err,
                    "Failed to send client approval response to controller"
                );
                Error::internal("Failed to send client approval response to controller")
            })?;

        ctx.actor_ref().unlink(&pending_approval.controller).await;

        Ok(())
    }

    #[message]
    pub(crate) async fn handle_sdk_client_list(
        &mut self,
    ) -> Result<Vec<(ProgramClient, ProgramClientMetadata)>, Error> {
        use crate::db::schema::{client_metadata, program_client};
        let mut conn = self.props.db.get().await?;

        let clients = program_client::table
            .inner_join(client_metadata::table)
            .select((
                ProgramClient::as_select(),
                ProgramClientMetadata::as_select(),
            ))
            .load::<(ProgramClient, ProgramClientMetadata)>(&mut conn)
            .await?;

        Ok(clients)
    }
}

#[messages]
impl OperatorSession {
    #[message]
    pub(crate) async fn handle_create_proposal(
        &mut self,
        kind: crate::db::models::ProposalKind,
        ttl_secs: Option<i64>,
    ) -> Result<i32, Error> {
        use crate::actors::proposal_manager::CreateProposal;
        let initiator_id = self.credentials.id;
        self.props
            .actors
            .proposal_manager
            .ask(CreateProposal { kind, initiator_id, ttl_secs })
            .await
            .map_err(|e| {
                error!(?e, "create_proposal failed");
                Error::internal("Failed to create proposal")
            })
    }

    #[message]
    pub(crate) async fn handle_cast_vote(
        &mut self,
        proposal_id: i32,
        approve: bool,
        signature: Vec<u8>,
    ) -> Result<crate::actors::proposal_manager::VoteOutcome, crate::actors::proposal_manager::Error> {
        use crate::actors::proposal_manager::CastVote;
        let operator_id = self.credentials.id;
        self.props
            .actors
            .proposal_manager
            .ask(CastVote { proposal_id, operator_id, approve, signature })
            .await
            .map_err(|err| match err {
                SendError::HandlerError(e) => e,
                _ => crate::actors::proposal_manager::Error::ExecutionFailed("actor unavailable".to_owned()),
            })
    }

    #[message]
    pub(crate) async fn handle_query_pending(
        &mut self,
    ) -> Vec<crate::actors::proposal_manager::ProposalSummary> {
        use crate::actors::proposal_manager::QueryPending;
        let operator_id = self.credentials.id;
        self.props
            .actors
            .proposal_manager
            .ask(QueryPending { operator_id })
            .await
            .unwrap_or_default()
    }
}

#[messages]
impl OperatorSession {
    #[message]
    pub(crate) async fn handle_contribute_rekey_passphrase(
        &mut self,
        passphrase: Vec<u8>,
    ) -> Result<bool, Error> {
        use crate::actors::vault_coordinator::ContributeRekey;
        use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};

        let operator_id = self.credentials.id;
        self.props
            .actors
            .vault_coordinator
            .ask(ContributeRekey {
                operator_id,
                passphrase: SafeCell::new(passphrase),
            })
            .await
            .map_err(|_| Error::internal("VaultCoordinator unavailable"))
    }

    #[message]
    pub(crate) async fn handle_contribute_recovery_rekey_passphrase(
        &mut self,
        recovery_operator_id: i32,
        passphrase: Vec<u8>,
    ) -> Result<bool, Error> {
        use crate::actors::vault_coordinator::ContributeRecoveryRekey;
        use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};

        self.props
            .actors
            .vault_coordinator
            .ask(ContributeRecoveryRekey {
                recovery_operator_id,
                passphrase: SafeCell::new(passphrase),
            })
            .await
            .map_err(|_| Error::internal("VaultCoordinator unavailable"))
    }
}
