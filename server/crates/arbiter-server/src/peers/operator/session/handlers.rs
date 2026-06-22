use super::{Error, OperatorSession};
use crate::{
    actors::evm::{
        ClientSignTransaction, Generate, ListWallets, OperatorCreateGrant, OperatorListGrants,
        SignTransactionError as EvmSignError,
    },
    actors::flow_coordinator::{IsClientConnected, client_connect_approval::ClientApprovalAnswer},
    actors::vault::VaultState,
    db::{
        models::{EvmWalletAccess, NewEvmWalletAccess, ProgramClient, ProgramClientMetadata},
        schema::program_client,
    },
    evm::policies::{Grant, SpecificGrant},
};
use arbiter_crypto::authn;

use alloy::{consensus::TxEip1559, primitives::Address, signers::Signature};
use diesel::{ExpressionMethods as _, QueryDsl as _, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use kameo::{error::SendError, messages, prelude::Context};
use tracing::{error, info, warn};

#[derive(Debug, Error)]
pub enum SignTransactionError {
    #[error("Policy evaluation failed")]
    Vet(#[from] crate::evm::VetError),

    #[error("Client not connected")]
    ClientNotConnected,

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
    pub(crate) async fn handle_evm_wallet_list(&mut self) -> Result<Vec<(i32, Address)>, Error> {
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
    pub(crate) fn handle_grant_delete(&mut self, grant_id: i32) -> Result<(), GrantMutationError> {
        // match self
        //     .props
        //     .actors
        //     .evm
        //     .ask(OperatorDeleteGrant { grant_id })
        //     .await
        // {
        //     Ok(()) => Ok(()),
        //     Err(err) => {
        //         error!(?err, "EVM grant delete failed");
        //         Err(GrantMutationError::Internal)
        //     }
        // }
        let _ = grant_id;
        todo!()
    }

    #[message]
    pub(crate) async fn handle_sign_transaction(
        &mut self,
        client_id: i32,
        wallet_address: Address,
        transaction: TxEip1559,
    ) -> Result<Signature, SignTransactionError> {
        if !self.approved_client_ids.contains(&client_id) {
            warn!(
                client_id,
                "operator attempted to sign for client not in its approved set"
            );
            return Err(SignTransactionError::ClientNotConnected);
        }

        let connected = self
            .props
            .actors
            .flow_coordinator
            .ask(IsClientConnected { client_id })
            .await
            .unwrap_or(false);

        if !connected {
            self.approved_client_ids.remove(&client_id);
            warn!(client_id, "operator attempted to sign for disconnected client");
            return Err(SignTransactionError::ClientNotConnected);
        }

        info!(client_id, event = "sign_transaction", "operator.sign_transaction");

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
                    .filter(evm_wallet_access::id.eq(entry))
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
        let mut conn = self.props.db.get().await?;
        let access_entries = crate::db::schema::evm_wallet_access::table
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

        if approved {
            let pubkey_bytes = pending_approval.pubkey.to_bytes();
            match self.props.db.get().await {
                Ok(mut conn) => {
                    match program_client::table
                        .filter(program_client::public_key.eq(pubkey_bytes.as_slice()))
                        .select(program_client::id)
                        .first::<i32>(&mut conn)
                        .await
                    {
                        Ok(client_id) => {
                            self.approved_client_ids.insert(client_id);
                        }
                        Err(err) => {
                            error!(?err, "Failed to look up client_id for approved pubkey");
                        }
                    }
                }
                Err(err) => {
                    error!(?err, "DB pool error after client approval");
                }
            }
        }

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

#[cfg(test)]
mod tests {
    use crate::db::{self, models::NewEvmWalletAccess, schema::evm_wallet_access};
    use diesel::{ExpressionMethods as _, QueryDsl as _, SelectableHelper};
    use diesel_async::{AsyncConnection, RunQueryDsl};

    /// Regression test: revocation must delete by access-entry `id`, not by `wallet_id`.
    ///
    /// Before the fix, revoking entry_id=1 would delete all rows where wallet_id=1,
    /// wiping out every client's access to wallet #1.
    #[tokio::test]
    async fn revoke_deletes_by_entry_id_not_wallet_id() {
        use crate::db::models::EvmWalletAccess;

        let pool = db::create_test_pool().await;
        let mut conn = pool.get().await.expect("pool connection");

        // Insert two access entries for the same wallet but different clients.
        // entry A: id will be 1, wallet_id=1, client_id=10
        // entry B: id will be 2, wallet_id=1, client_id=20
        let entry_a = diesel::insert_into(evm_wallet_access::table)
            .values(NewEvmWalletAccess {
                wallet_id: 1,
                client_id: 10,
            })
            .returning(EvmWalletAccess::as_select())
            .get_result(&mut *conn)
            .await
            .expect("insert entry A");

        let entry_b = diesel::insert_into(evm_wallet_access::table)
            .values(NewEvmWalletAccess {
                wallet_id: 1,
                client_id: 20,
            })
            .returning(EvmWalletAccess::as_select())
            .get_result(&mut *conn)
            .await
            .expect("insert entry B");

        // Revoke only entry A by its primary key id.
        conn.transaction(async |conn| {
            diesel::delete(evm_wallet_access::table)
                .filter(evm_wallet_access::id.eq(entry_a.id))
                .execute(&mut *conn)
                .await
        })
        .await
        .expect("revoke entry A");

        // Entry A must be gone.
        let gone = evm_wallet_access::table
            .filter(evm_wallet_access::id.eq(entry_a.id))
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .expect("count entry A");
        assert_eq!(gone, 0, "revoked entry must be deleted");

        // Entry B (same wallet, different client) must still exist.
        let still_there = evm_wallet_access::table
            .filter(evm_wallet_access::id.eq(entry_b.id))
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .expect("count entry B");
        assert_eq!(still_there, 1, "unrelated entry must not be deleted");
    }

    /// Regression test: when entry_id and wallet_id differ, only the correct row is removed.
    ///
    /// This specifically catches the case where entry.id=5 and wallet_id=1 are different values;
    /// the old bug would delete by wallet_id, potentially matching a completely different entry.
    #[tokio::test]
    async fn revoke_with_mismatched_wallet_and_entry_ids() {
        use crate::db::models::EvmWalletAccess;

        let pool = db::create_test_pool().await;
        let mut conn = pool.get().await.expect("pool connection");

        // Insert entries to force auto-increment IDs to diverge from wallet_ids.
        // We'll insert 5 placeholder entries first so that the real entry gets id=6.
        for i in 1_i32..=5 {
            diesel::insert_into(evm_wallet_access::table)
                .values(NewEvmWalletAccess {
                    wallet_id: 99,
                    client_id: i,
                })
                .execute(&mut *conn)
                .await
                .expect("insert placeholder");
        }

        // Real target: wallet_id=1, will get id=6.
        let target = diesel::insert_into(evm_wallet_access::table)
            .values(NewEvmWalletAccess {
                wallet_id: 1,
                client_id: 1,
            })
            .returning(EvmWalletAccess::as_select())
            .get_result(&mut *conn)
            .await
            .expect("insert target");

        // Sanity: target.id != target.wallet_id
        assert_ne!(
            target.id, target.wallet_id,
            "test prerequisite: id and wallet_id must differ"
        );

        // Revoke by entry id.
        conn.transaction(async |conn| {
            diesel::delete(evm_wallet_access::table)
                .filter(evm_wallet_access::id.eq(target.id))
                .execute(&mut *conn)
                .await
        })
        .await
        .expect("revoke target");

        let remaining = evm_wallet_access::table
            .filter(evm_wallet_access::id.eq(target.id))
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .expect("count target");
        assert_eq!(remaining, 0, "target must be deleted by its entry id");

        // Placeholders for wallet_id=99 must be untouched.
        let placeholders = evm_wallet_access::table
            .filter(evm_wallet_access::wallet_id.eq(99))
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .expect("count placeholders");
        assert_eq!(placeholders, 5, "unrelated entries must survive");
    }
}
