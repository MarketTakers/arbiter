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
        grant_wallet_access(&mut conn, entries).await?;
        Ok(())
    }

    /// A revoke that matched fewer rows than it named did not do what the operator asked:
    /// the id was never granted, or someone revoked it first. Answering `Ok` there tells the
    /// operator access is cut off when nothing changed. The rows that did match stay revoked
    /// -- rolling them back to report the shortfall would leave live access behind.
    #[message]
    pub(crate) async fn handle_revoke_evm_wallet_access(
        &mut self,
        entries: Vec<i32>,
    ) -> Result<(), Error> {
        let mut conn = self.props.db.get().await?;
        let revoked = revoke_wallet_access(&mut conn, &entries).await?;
        if revoked != entries.len() {
            return Err(Error::PartialRevoke {
                requested: entries.len(),
                revoked,
            });
        }
        Ok(())
    }

    #[message]
    pub(crate) async fn handle_list_wallet_access(
        &mut self,
    ) -> Result<Vec<EvmWalletAccess>, Error> {
        use crate::db::schema::evm_wallet_access;
        let mut conn = self.props.db.get().await?;
        let access_entries = evm_wallet_access::table
            .filter(evm_wallet_access::revoked_at.is_null())
            .select(EvmWalletAccess::as_select())
            .load::<_>(&mut conn)
            .await?;
        Ok(access_entries)
    }
}

/// Grants access, reviving a previously revoked row rather than leaving it shadowed:
/// `uniq_wallet_access` is a unique index on `(wallet_id, client_id)`, so a plain insert
/// would conflict forever on a row that was revoked but never deleted.
///
/// Reviving restores visibility and nothing else: [`revoke_wallet_access`] closes the grants
/// that hung off the access, so a persistent grant takes its own vote again (§3.2).
pub(crate) async fn grant_wallet_access(
    conn: &mut crate::db::DatabaseConnection,
    entries: Vec<NewEvmWalletAccess>,
) -> Result<(), diesel::result::Error> {
    use crate::db::{models::SqliteTimestamp, schema::evm_wallet_access};

    conn.transaction(async |conn| {
        for entry in entries {
            diesel::insert_into(evm_wallet_access::table)
                .values(&entry)
                .on_conflict((evm_wallet_access::wallet_id, evm_wallet_access::client_id))
                .do_update()
                .set(evm_wallet_access::revoked_at.eq(None::<SqliteTimestamp>))
                .execute(&mut *conn)
                .await?;
        }

        Ok(())
    })
    .await
}

/// Marks access rows revoked by their own id rather than deleting them, and revokes every
/// grant that hangs off them. Returns how many access rows this call revoked.
///
/// The wire carries `WalletAccessEntry.id` values, so filtering by `wallet_id` here would
/// revoke every client's access to that wallet. Deleting is not an option: `evm_basic_grant`,
/// `evm_transaction_log`, and `proposal_persistent_grant` all reference this row
/// `on delete restrict`, so an access that was ever granted, signed with, or proposed
/// against can never be deleted -- only marked revoked.
///
/// The dependent grants have to go with it. `grant_wallet_access` revives a revoked row by
/// its id, and grant lookup keys on `wallet_access_id` alone, so leaving the grants live
/// would make a later re-grant restore every persistent grant the access ever held, with its
/// original volume and rate limits. §3.2 votes visibility and a persistent grant separately;
/// a committee that approves visibility must not silently hand back signing authority it did
/// not vote on. The filter names every requested id, not just the rows this call flipped, so
/// an access revoked before this fix has its orphaned grants closed too.
pub(crate) async fn revoke_wallet_access(
    conn: &mut crate::db::DatabaseConnection,
    ids: &[i32],
) -> Result<usize, diesel::result::Error> {
    use crate::db::{
        models::SqliteTimestamp,
        schema::{evm_basic_grant, evm_wallet_access},
    };

    conn.transaction(async |conn| {
        let now = SqliteTimestamp::now();

        let revoked = diesel::update(evm_wallet_access::table)
            .filter(evm_wallet_access::id.eq_any(ids))
            .filter(evm_wallet_access::revoked_at.is_null())
            .set(evm_wallet_access::revoked_at.eq(now.clone()))
            .execute(&mut *conn)
            .await?;

        diesel::update(evm_basic_grant::table)
            .filter(evm_basic_grant::wallet_access_id.eq_any(ids))
            .filter(evm_basic_grant::revoked_at.is_null())
            .set(evm_basic_grant::revoked_at.eq(now))
            .execute(&mut *conn)
            .await?;

        Ok(revoked)
    })
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
        let initiator_id = OperatorIdentityId::from_raw(self.ordinary_id()?);
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
        let operator_id = OperatorIdentityId::from_raw(
            self.ordinary_id()
                .map_err(|_| crate::actors::proposal_manager::Error::NotAllowedForRecoveryOperator)?,
        );
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
        let Ok(id) = self.ordinary_id() else {
            // The pending list is per ordinary operator; a recovery operator has no view of it.
            return Vec::new();
        };
        let operator_id = OperatorIdentityId::from_raw(id);
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

        let operator_id = self.ordinary_id()?;
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

    /// §3.3: a re-key refreshes every share, recovery shares included, so a recovery operator
    /// has one to contribute here.
    ///
    /// It cannot reach this handler yet: `peers::operator::start` refuses a recovery peer an
    /// operator session, because a session carries the whole ordinary-governance surface that
    /// §3.5 keeps out of a recovery operator's hands. Until a recovery-scoped session exists,
    /// this refuses every caller -- which is the safe direction, and the id it would use comes
    /// from the handshake either way.
    #[message]
    pub(crate) async fn handle_contribute_recovery_rekey_passphrase(
        &mut self,
        passphrase: Vec<u8>,
    ) -> Result<bool, Error> {
        use crate::actors::vault_coordinator::ContributeRecoveryRekey;
        use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};

        let recovery_operator_id = self.recovery_id()?;
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
    use super::{grant_wallet_access, revoke_wallet_access};
    use crate::db::{self, models, schema};

    use diesel::{ExpressionMethods as _, QueryDsl as _, dsl::insert_into};
    use diesel_async::RunQueryDsl;

    /// Inserts a fresh root key, an aead-encrypted wallet secret, and the wallet itself.
    /// Returns the wallet's id and its 20-byte address.
    async fn seed_wallet(conn: &mut db::DatabaseConnection) -> (models::EvmWalletId, Vec<u8>) {
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

        let address = rand::random::<[u8; 20]>().to_vec();
        let wallet_id: models::EvmWalletId = insert_into(schema::evm_wallet::table)
            .values((
                schema::evm_wallet::address.eq(address.clone()),
                schema::evm_wallet::aead_encrypted_id.eq(aead_id),
            ))
            .returning(schema::evm_wallet::id)
            .get_result(conn)
            .await
            .unwrap();

        (wallet_id, address)
    }

    async fn seed_client_metadata(conn: &mut db::DatabaseConnection) -> i32 {
        insert_into(schema::client_metadata::table)
            .values(schema::client_metadata::name.eq("test"))
            .returning(schema::client_metadata::id)
            .get_result(conn)
            .await
            .unwrap()
    }

    /// Inserts a `program_client` row under the given `client_metadata` row, keyed by its
    /// own random public key.
    async fn seed_client(conn: &mut db::DatabaseConnection, metadata_id: i32) -> i32 {
        insert_into(schema::program_client::table)
            .values((
                schema::program_client::public_key.eq(rand::random::<[u8; 32]>().to_vec()),
                schema::program_client::metadata_id.eq(metadata_id),
            ))
            .returning(schema::program_client::id)
            .get_result(conn)
            .await
            .unwrap()
    }

    /// Inserts an access row directly, for fixtures that need one to already exist.
    /// Production code grants access through [`grant_wallet_access`].
    async fn insert_wallet_access(
        conn: &mut db::DatabaseConnection,
        wallet_id: models::EvmWalletId,
        client_id: i32,
    ) -> i32 {
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

    /// The grants that grant lookup would treat as live for this access: the exact filter
    /// `EtherTransfer::try_find_grant` and `TokenTransfer::try_find_grant` apply.
    async fn live_grants_for(conn: &mut db::DatabaseConnection, access_id: i32) -> Vec<i32> {
        schema::evm_basic_grant::table
            .filter(schema::evm_basic_grant::wallet_access_id.eq(access_id))
            .filter(schema::evm_basic_grant::revoked_at.is_null())
            .select(schema::evm_basic_grant::id)
            .load(conn)
            .await
            .unwrap()
    }

    /// Two clients share one wallet. Revoking one access row must leave the other alone,
    /// and must mark the row revoked rather than deleting it.
    #[tokio::test]
    async fn revoking_one_access_leaves_the_other_client_alone() {
        let pool = db::create_test_pool().await;
        let mut conn = pool.get().await.unwrap();

        let (wallet_id, _address) = seed_wallet(&mut conn).await;
        let metadata_id = seed_client_metadata(&mut conn).await;
        let first_client = seed_client(&mut conn, metadata_id).await;
        let second_client = seed_client(&mut conn, metadata_id).await;

        let first_access = insert_wallet_access(&mut conn, wallet_id, first_client).await;
        let _second_access = insert_wallet_access(&mut conn, wallet_id, second_client).await;

        let removed = revoke_wallet_access(&mut conn, &[first_access])
            .await
            .unwrap();
        assert_eq!(removed, 1);

        let active: Vec<i32> = schema::evm_wallet_access::table
            .filter(schema::evm_wallet_access::revoked_at.is_null())
            .select(schema::evm_wallet_access::client_id)
            .load(&mut conn)
            .await
            .unwrap();
        assert_eq!(
            active,
            vec![second_client],
            "revoking one access row removed another client's access"
        );

        let total: i64 = schema::evm_wallet_access::table
            .count()
            .get_result(&mut conn)
            .await
            .unwrap();
        assert_eq!(
            total, 2,
            "revoking an access row must mark it revoked, not delete it"
        );

        // The count `handle_revoke_evm_wallet_access` answers on: a second revoke of the same
        // id, like a revoke of an id that never existed, changes nothing and must say so.
        let again = revoke_wallet_access(&mut conn, &[first_access])
            .await
            .unwrap();
        assert_eq!(again, 0, "an already revoked access must report no rows");
    }

    /// The bug this round fixes: once an access has been used for a grant, a signed
    /// transaction, or a proposed persistent grant, three tables reference
    /// `evm_wallet_access` `on delete restrict`, so deleting the row is no longer possible
    /// once foreign keys are enforced. Revoking must still succeed by marking it revoked.
    #[tokio::test]
    async fn revoking_an_access_with_grant_log_and_proposal_succeeds() {
        use crate::db::proposal::{Proposal as _, persistent_grant, persistent_grant::PersistentGrant};

        let pool = db::create_test_pool().await;
        let mut conn = pool.get().await.unwrap();

        let (wallet_id, _address) = seed_wallet(&mut conn).await;
        let metadata_id = seed_client_metadata(&mut conn).await;
        let client_id = seed_client(&mut conn, metadata_id).await;
        let access_id = insert_wallet_access(&mut conn, wallet_id, client_id).await;

        // A grant against this access...
        let grant_id: i32 = insert_into(schema::evm_basic_grant::table)
            .values(models::NewEvmBasicGrant {
                wallet_access_id: access_id,
                chain_id: 1u64.into(),
                valid_from: None,
                valid_until: None,
                max_gas_fee_per_gas: None,
                max_priority_fee_per_gas: None,
                rate_limit_count: None,
                rate_limit_window_secs: None,
                revoked_at: None,
            })
            .returning(schema::evm_basic_grant::id)
            .get_result(&mut conn)
            .await
            .unwrap();

        // ...a signed transaction against that grant...
        insert_into(schema::evm_transaction_log::table)
            .values(models::NewEvmTransactionLog {
                grant_id,
                wallet_access_id: access_id,
                chain_id: 1u64.into(),
                eth_value: vec![0u8; 32],
                signed_at: models::SqliteTimestamp(chrono::Utc::now()),
            })
            .execute(&mut conn)
            .await
            .unwrap();

        // ...and a persistent-grant proposal that named this access before it was voted on.
        let operator_id: models::OperatorIdentityId =
            insert_into(schema::operator_identity::table)
                .values(schema::operator_identity::public_key.eq(rand::random::<[u8; 32]>().to_vec()))
                .returning(schema::operator_identity::id)
                .get_result(&mut conn)
                .await
                .unwrap();
        let proposal_id: models::ProposalId = insert_into(schema::proposal::table)
            .values(&models::NewProposal {
                kind: db::proposal::ProposalKindTag::ApprovePersistentGrant,
                initiator_id: operator_id,
                expires_at: models::SqliteTimestamp(chrono::Utc::now() + chrono::Duration::days(1)),
            })
            .returning(schema::proposal::id)
            .get_result(&mut conn)
            .await
            .unwrap();
        PersistentGrant::insert(
            proposal_id,
            &persistent_grant::Settings {
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
            },
            &mut conn,
        )
        .await
        .unwrap();

        // Before this round's fix, this would fail with a foreign-key violation.
        let removed = revoke_wallet_access(&mut conn, &[access_id]).await.unwrap();
        assert_eq!(removed, 1);

        let revoked_at: Option<models::SqliteTimestamp> = schema::evm_wallet_access::table
            .find(access_id)
            .select(schema::evm_wallet_access::revoked_at)
            .first(&mut conn)
            .await
            .unwrap();
        assert!(
            revoked_at.is_some(),
            "the row must be marked revoked, not deleted"
        );
    }

    /// A revoked access must no longer resolve through the lookup `shared_analyze_transaction`
    /// and `client_sign_transaction` share -- otherwise the SDK client keeps signing after
    /// the operator believes it has been cut off.
    #[tokio::test]
    async fn revoked_access_no_longer_authorizes_signing() {
        use crate::actors::{
            GlobalActors,
            evm::{EvmActor, SignTransactionError},
            vault::Vault,
        };
        use alloy::{
            consensus::TxEip1559,
            eips::eip2930::AccessList,
            primitives::{Address, Bytes, TxKind, U256},
        };
        use kameo::actor::Spawn as _;

        let pool = db::create_test_pool().await;
        let mut conn = pool.get().await.unwrap();

        let (wallet_id, address) = seed_wallet(&mut conn).await;
        let metadata_id = seed_client_metadata(&mut conn).await;
        let client_id = seed_client(&mut conn, metadata_id).await;
        let access_id = insert_wallet_access(&mut conn, wallet_id, client_id).await;
        drop(conn);

        let vault = Vault::spawn(
            Vault::new(pool.clone(), GlobalActors::spawn_message_bus())
                .await
                .unwrap(),
        );
        let mut evm_actor = EvmActor::new(vault, pool.clone());

        let transaction = TxEip1559 {
            chain_id: 1,
            nonce: 0,
            gas_limit: 21_000,
            max_fee_per_gas: 0,
            max_priority_fee_per_gas: 0,
            to: TxKind::Call(Address::ZERO),
            value: U256::ZERO,
            input: Bytes::new(),
            access_list: AccessList::default(),
        };
        let wallet_address = Address::from_slice(&address);

        // The paired positive case: while the access stands, both lookups resolve it and the
        // calls fail further along (no grant, sealed vault) rather than at the access filter.
        // Without this, a filter that rejected every row would pass the assertions below.
        let live_analyze = evm_actor
            .shared_analyze_transaction(client_id, wallet_address, transaction.clone())
            .await;
        assert!(
            !matches!(live_analyze, Err(SignTransactionError::WalletNotFound)),
            "a live access must resolve through shared_analyze_transaction: {live_analyze:?}"
        );
        let live_sign = evm_actor
            .client_sign_transaction(client_id, wallet_address, transaction.clone())
            .await;
        assert!(
            !matches!(live_sign, Err(SignTransactionError::WalletNotFound)),
            "a live access must resolve through client_sign_transaction: {live_sign:?}"
        );

        let mut conn = pool.get().await.unwrap();
        revoke_wallet_access(&mut conn, &[access_id]).await.unwrap();
        drop(conn);

        // Both lookups resolve access the same way; both must reject the revoked row before
        // ever touching the vault (neither call bootstraps one).
        let analyze_result = evm_actor
            .shared_analyze_transaction(client_id, wallet_address, transaction.clone())
            .await;
        assert!(
            matches!(analyze_result, Err(SignTransactionError::WalletNotFound)),
            "a revoked access must not authorize shared_analyze_transaction: {analyze_result:?}"
        );

        let sign_result = evm_actor
            .client_sign_transaction(client_id, wallet_address, transaction)
            .await;
        assert!(
            matches!(sign_result, Err(SignTransactionError::WalletNotFound)),
            "a revoked access must not authorize client_sign_transaction: {sign_result:?}"
        );
    }

    /// Re-granting a revoked access must restore it rather than silently doing nothing:
    /// `uniq_wallet_access` is a unique index on `(wallet_id, client_id)`, so a plain insert
    /// would conflict on the revoked row forever.
    #[tokio::test]
    async fn regranting_a_revoked_access_restores_it() {
        let pool = db::create_test_pool().await;
        let mut conn = pool.get().await.unwrap();

        let (wallet_id, _address) = seed_wallet(&mut conn).await;
        let metadata_id = seed_client_metadata(&mut conn).await;
        let client_id = seed_client(&mut conn, metadata_id).await;
        let access_id = insert_wallet_access(&mut conn, wallet_id, client_id).await;

        revoke_wallet_access(&mut conn, &[access_id]).await.unwrap();

        grant_wallet_access(
            &mut conn,
            vec![models::NewEvmWalletAccess {
                wallet_id,
                client_id,
            }],
        )
        .await
        .unwrap();

        let revoked_at: Option<models::SqliteTimestamp> = schema::evm_wallet_access::table
            .find(access_id)
            .select(schema::evm_wallet_access::revoked_at)
            .first(&mut conn)
            .await
            .unwrap();
        assert!(
            revoked_at.is_none(),
            "re-granting a revoked access must clear revoked_at"
        );

        let total: i64 = schema::evm_wallet_access::table
            .count()
            .get_result(&mut conn)
            .await
            .unwrap();
        assert_eq!(
            total, 1,
            "re-granting a revoked access must revive the existing row, not add a second one"
        );
    }

    /// §3.2 puts wallet visibility and a persistent grant to two separate votes. Reviving a
    /// revoked access restores visibility, and must restore nothing else: grant lookup keys on
    /// `wallet_access_id` with `revoked_at is null`, so a grant left open when the access was
    /// cut off would come back live -- with its original volume and rate limits -- the moment
    /// the id revives. `EvmActor::grant_wallet_access` executes an approved `GrantWalletAccess`
    /// proposal, so that would hand signing authority back to a committee that voted only on
    /// visibility.
    #[tokio::test]
    async fn regranting_an_access_does_not_revive_its_grants() {
        let pool = db::create_test_pool().await;
        let mut conn = pool.get().await.unwrap();

        let (wallet_id, _address) = seed_wallet(&mut conn).await;
        let metadata_id = seed_client_metadata(&mut conn).await;
        let client_id = seed_client(&mut conn, metadata_id).await;

        let entry = || models::NewEvmWalletAccess {
            wallet_id,
            client_id,
        };
        grant_wallet_access(&mut conn, vec![entry()]).await.unwrap();
        let access_id: i32 = schema::evm_wallet_access::table
            .filter(schema::evm_wallet_access::wallet_id.eq(wallet_id))
            .filter(schema::evm_wallet_access::client_id.eq(client_id))
            .select(schema::evm_wallet_access::id)
            .first(&mut conn)
            .await
            .unwrap();

        // The row every persistent grant hangs off: the specific ether- or token-transfer
        // rows reference it, so liveness is decided here.
        let grant_id: i32 = insert_into(schema::evm_basic_grant::table)
            .values(models::NewEvmBasicGrant {
                wallet_access_id: access_id,
                chain_id: 1u64.into(),
                valid_from: None,
                valid_until: None,
                max_gas_fee_per_gas: None,
                max_priority_fee_per_gas: None,
                rate_limit_count: None,
                rate_limit_window_secs: None,
                revoked_at: None,
            })
            .returning(schema::evm_basic_grant::id)
            .get_result(&mut conn)
            .await
            .unwrap();

        assert_eq!(
            live_grants_for(&mut conn, access_id).await,
            vec![grant_id],
            "the seeded grant must start out live, or the assertions below prove nothing"
        );

        revoke_wallet_access(&mut conn, &[access_id]).await.unwrap();
        assert!(
            live_grants_for(&mut conn, access_id).await.is_empty(),
            "revoking an access must revoke the grants that hang off it"
        );

        grant_wallet_access(&mut conn, vec![entry()]).await.unwrap();

        let revoked_at: Option<models::SqliteTimestamp> = schema::evm_wallet_access::table
            .find(access_id)
            .select(schema::evm_wallet_access::revoked_at)
            .first(&mut conn)
            .await
            .unwrap();
        assert!(
            revoked_at.is_none(),
            "re-granting must restore visibility for the access itself"
        );
        assert!(
            live_grants_for(&mut conn, access_id).await.is_empty(),
            "re-granting an access must not revive the grants it held before revocation"
        );
    }
}
