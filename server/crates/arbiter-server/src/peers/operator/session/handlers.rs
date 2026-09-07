use super::{Error, OperatorSession};
use crate::db::models::{OperatorIdentityId, ProposalId};
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
        revoke_wallet_access(&mut conn, &entries).await?;
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

/// Deletes access rows by their own id. The wire carries `WalletAccessEntry.id` values, so
/// filtering by `wallet_id` here would revoke every client's access to that wallet.
pub(crate) async fn revoke_wallet_access(
    conn: &mut crate::db::DatabaseConnection,
    ids: &[i32],
) -> Result<usize, diesel::result::Error> {
    use crate::db::schema::evm_wallet_access;

    diesel::delete(evm_wallet_access::table)
        .filter(evm_wallet_access::id.eq_any(ids))
        .execute(conn)
        .await
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
        kind: crate::db::proposal::ProposalKind,
        ttl_secs: Option<u32>,
    ) -> Result<ProposalId, Error> {
        use crate::actors::proposal_manager::CreateProposal;
        let initiator_id = OperatorIdentityId::from_raw(self.credentials.id);
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
        proposal_id: ProposalId,
        approve: bool,
        signature: Vec<u8>,
    ) -> Result<crate::actors::proposal_manager::VoteOutcome, crate::actors::proposal_manager::Error> {
        use crate::actors::proposal_manager::CastVote;
        let operator_id = OperatorIdentityId::from_raw(self.credentials.id);
        self.props
            .actors
            .proposal_manager
            .ask(CastVote { proposal_id, operator_id, approve, signature })
            .await
            .map_err(|err| match err {
                SendError::HandlerError(e) => e,
                _ => crate::actors::proposal_manager::Error::Unavailable,
            })
    }

    #[message]
    pub(crate) async fn handle_query_pending(
        &mut self,
    ) -> Vec<crate::actors::proposal_manager::ProposalSummary> {
        use crate::actors::proposal_manager::QueryPending;
        let operator_id = OperatorIdentityId::from_raw(self.credentials.id);
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

#[cfg(test)]
mod tests {
    use super::revoke_wallet_access;
    use crate::db::{self, models, schema};

    use diesel::{ExpressionMethods as _, QueryDsl as _, dsl::insert_into};
    use diesel_async::RunQueryDsl;

    /// Two clients share one wallet. Revoking one access row must leave the other alone.
    #[tokio::test]
    async fn revoking_one_access_leaves_the_other_client_alone() {
        let pool = db::create_test_pool().await;
        let mut conn = pool.get().await.unwrap();

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
            .get_result(&mut conn)
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
            .get_result(&mut conn)
            .await
            .unwrap();

        let wallet_id: models::EvmWalletId = insert_into(schema::evm_wallet::table)
            .values((
                schema::evm_wallet::address.eq(vec![0u8; 20]),
                schema::evm_wallet::aead_encrypted_id.eq(aead_id),
            ))
            .returning(schema::evm_wallet::id)
            .get_result(&mut conn)
            .await
            .unwrap();

        let metadata_id: i32 = insert_into(schema::client_metadata::table)
            .values(schema::client_metadata::name.eq("test"))
            .returning(schema::client_metadata::id)
            .get_result(&mut conn)
            .await
            .unwrap();

        let first_client: i32 = insert_into(schema::program_client::table)
            .values((
                schema::program_client::public_key.eq(vec![1u8; 32]),
                schema::program_client::metadata_id.eq(metadata_id),
            ))
            .returning(schema::program_client::id)
            .get_result(&mut conn)
            .await
            .unwrap();

        let second_client: i32 = insert_into(schema::program_client::table)
            .values((
                schema::program_client::public_key.eq(vec![2u8; 32]),
                schema::program_client::metadata_id.eq(metadata_id),
            ))
            .returning(schema::program_client::id)
            .get_result(&mut conn)
            .await
            .unwrap();

        let first_access: i32 = insert_into(schema::evm_wallet_access::table)
            .values((
                schema::evm_wallet_access::wallet_id.eq(wallet_id),
                schema::evm_wallet_access::client_id.eq(first_client),
            ))
            .returning(schema::evm_wallet_access::id)
            .get_result(&mut conn)
            .await
            .unwrap();

        let _second_access: i32 = insert_into(schema::evm_wallet_access::table)
            .values((
                schema::evm_wallet_access::wallet_id.eq(wallet_id),
                schema::evm_wallet_access::client_id.eq(second_client),
            ))
            .returning(schema::evm_wallet_access::id)
            .get_result(&mut conn)
            .await
            .unwrap();

        let removed = revoke_wallet_access(&mut conn, &[first_access])
            .await
            .unwrap();
        assert_eq!(removed, 1);

        let survivors: Vec<i32> = schema::evm_wallet_access::table
            .select(schema::evm_wallet_access::client_id)
            .load(&mut conn)
            .await
            .unwrap();
        assert_eq!(
            survivors,
            vec![second_client],
            "revoking one access row removed another client's access"
        );
    }
}
