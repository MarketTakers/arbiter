use crate::{
    actors::{
        evm::EvmActor,
        vault::Vault,
        vault_coordinator::{StartRekey, VaultCoordinator},
    },
    db::{
        self,
        models::{
            NewProposal, NewProposalVote, NewRecoveryProposalVote, NewRecoveryWakeupRequest,
            Proposal, ProposalStatus, SqliteTimestamp,
        },
        schema,
    },
};
use chrono::Utc;
use diesel::{ExpressionMethods as _, QueryDsl};
use diesel_async::RunQueryDsl;
use kameo::{actor::ActorRef, messages};
use strum::{Display, EnumString, IntoStaticStr};
use tracing::{error, warn};

pub const DEFAULT_TTL_SECS: i64 = 7 * 24 * 60 * 60; // 7 days

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum ProposalKindTag {
    ApproveSdkClient,
    GrantWalletAccess,
    ApproveServerUpdate,
    ReplaceOperator,
    UpdateShamirParameters,
    ApprovePersistentGrant,
    ApproveOneOffTransaction,
}

#[derive(Debug, Clone)]
pub enum ProposalKind {
    ApproveSdkClient {
        client_id: i32,
    },
    GrantWalletAccess {
        wallet_id: i32,
        client_id: i32,
    },
    ApproveServerUpdate,
    ReplaceOperator {
        old_operator_id: i32,
        new_pubkey: Vec<u8>,
    },
    UpdateShamirParameters {
        new_n: u8,
    },
    ApprovePersistentGrant {
        payload_bytes: Vec<u8>,
    },
    ApproveOneOffTransaction {
        payload_bytes: Vec<u8>,
    },
}

impl ProposalKind {
    pub const fn tag(&self) -> ProposalKindTag {
        match self {
            Self::ApproveSdkClient { .. } => ProposalKindTag::ApproveSdkClient,
            Self::GrantWalletAccess { .. } => ProposalKindTag::GrantWalletAccess,
            Self::ApproveServerUpdate => ProposalKindTag::ApproveServerUpdate,
            Self::ReplaceOperator { .. } => ProposalKindTag::ReplaceOperator,
            Self::UpdateShamirParameters { .. } => ProposalKindTag::UpdateShamirParameters,
            Self::ApprovePersistentGrant { .. } => ProposalKindTag::ApprovePersistentGrant,
            Self::ApproveOneOffTransaction { .. } => ProposalKindTag::ApproveOneOffTransaction,
        }
    }

    pub fn kind_str(&self) -> &'static str {
        self.tag().into()
    }

    pub fn encode_payload(&self) -> Vec<u8> {
        match self {
            Self::ApproveSdkClient { client_id } => client_id.to_be_bytes().to_vec(),
            Self::GrantWalletAccess {
                wallet_id,
                client_id,
            } => {
                let mut buf = Vec::with_capacity(8);
                buf.extend_from_slice(&wallet_id.to_be_bytes());
                buf.extend_from_slice(&client_id.to_be_bytes());
                buf
            }
            Self::ApproveServerUpdate => vec![],
            Self::ReplaceOperator {
                old_operator_id,
                new_pubkey,
            } => {
                let len = u32::try_from(new_pubkey.len()).expect("pubkey len fits in u32");
                let mut buf = Vec::with_capacity(4 + 4 + new_pubkey.len());
                buf.extend_from_slice(&old_operator_id.to_be_bytes());
                buf.extend_from_slice(&len.to_be_bytes());
                buf.extend_from_slice(new_pubkey);
                buf
            }
            Self::UpdateShamirParameters { new_n } => vec![*new_n],
            Self::ApprovePersistentGrant { payload_bytes }
            | Self::ApproveOneOffTransaction { payload_bytes } => payload_bytes.clone(),
        }
    }

    /// Key-rotation proposals require every operator to approve (§3.3).
    #[must_use]
    pub fn requires_full_quorum(kind: &str) -> bool {
        matches!(
            kind.parse::<ProposalKindTag>(),
            Ok(ProposalKindTag::ReplaceOperator | ProposalKindTag::UpdateShamirParameters)
        )
    }

    pub fn decode(kind: &str, payload: &[u8]) -> Result<Self, String> {
        let tag = kind
            .parse::<ProposalKindTag>()
            .map_err(|_| format!("unknown proposal kind: {kind}"))?;
        match tag {
            ProposalKindTag::ApproveSdkClient => {
                let bytes = <[u8; 4]>::try_from(payload)
                    .map_err(|_| "invalid payload for approve_sdk_client".to_owned())?;
                Ok(Self::ApproveSdkClient {
                    client_id: i32::from_be_bytes(bytes),
                })
            }
            ProposalKindTag::GrantWalletAccess => {
                let bytes = <[u8; 8]>::try_from(payload)
                    .map_err(|_| "invalid payload for grant_wallet_access".to_owned())?;
                Ok(Self::GrantWalletAccess {
                    wallet_id: i32::from_be_bytes(bytes[..4].try_into().unwrap()),
                    client_id: i32::from_be_bytes(bytes[4..].try_into().unwrap()),
                })
            }
            ProposalKindTag::ApproveServerUpdate => Ok(Self::ApproveServerUpdate),
            ProposalKindTag::ReplaceOperator => {
                let (id_bytes, rest) = payload
                    .split_first_chunk::<4>()
                    .ok_or_else(|| "replace_operator payload too short".to_owned())?;
                let old_operator_id = i32::from_be_bytes(*id_bytes);
                let (len_bytes, rest) = rest
                    .split_first_chunk::<4>()
                    .ok_or_else(|| "replace_operator payload too short".to_owned())?;
                let len = u32::from_be_bytes(*len_bytes);
                let len = usize::try_from(len).unwrap_or(usize::MAX);
                let new_pubkey = rest
                    .get(..len)
                    .ok_or_else(|| "replace_operator payload truncated".to_owned())?
                    .to_vec();
                Ok(Self::ReplaceOperator {
                    old_operator_id,
                    new_pubkey,
                })
            }
            ProposalKindTag::UpdateShamirParameters => {
                let &[new_n] = payload else {
                    return Err("invalid payload for update_shamir_parameters".to_owned());
                };
                Ok(Self::UpdateShamirParameters { new_n })
            }
            ProposalKindTag::ApprovePersistentGrant => Ok(Self::ApprovePersistentGrant {
                payload_bytes: payload.to_vec(),
            }),
            ProposalKindTag::ApproveOneOffTransaction => Ok(Self::ApproveOneOffTransaction {
                payload_bytes: payload.to_vec(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteOutcome {
    Pending,
    QuorumApproved,
    QuorumRejected,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Proposal not found")]
    ProposalNotFound,
    #[error("Proposal is not pending")]
    ProposalNotPending,
    #[error("Operator already voted on this proposal")]
    AlreadyVoted,
    #[error("Invalid vote signature")]
    InvalidSignature,
    #[error("Operator not found")]
    OperatorNotFound,
    #[error("Database connection error: {0}")]
    DatabaseConnection(#[from] db::PoolError),
    #[error("Database query error: {0}")]
    DatabaseQuery(#[from] diesel::result::Error),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Recovery operators are sleeping")]
    RecoveryNotActive,
    #[error("Recovery operators may only vote on operator replacement")]
    NotAllowedForRecoveryOperator,
    #[error("A recovery wake-up is already pending or active")]
    WakeupAlreadyPending,
    #[error("No active recovery wake-up to cancel")]
    NoActiveWakeup,
}

#[derive(Debug)]
pub struct ProposalSummary {
    pub id: i32,
    pub kind: String,
    pub initiator_id: i32,
    pub expires_at: SqliteTimestamp,
    pub approve_count: i64,
    pub reject_count: i64,
}

pub struct ProposalManager {
    pub(crate) db: db::DatabasePool,
    pub(crate) vault: ActorRef<Vault>,
    pub(crate) evm: ActorRef<EvmActor>,
    pub(crate) vault_coordinator: ActorRef<VaultCoordinator>,
}

impl ProposalManager {
    pub const fn new(
        db: db::DatabasePool,
        vault: ActorRef<Vault>,
        evm: ActorRef<EvmActor>,
        vault_coordinator: ActorRef<VaultCoordinator>,
    ) -> Self {
        Self {
            db,
            vault,
            evm,
            vault_coordinator,
        }
    }
}

impl kameo::Actor for ProposalManager {
    type Args = Self;
    type Error = ();

    async fn on_start(args: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        let weak = actor_ref.downgrade();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_hours(1)).await;
                match weak.upgrade() {
                    Some(r) => {
                        let _ = r.ask(ExpireStale).await;
                    }
                    None => break,
                }
            }
        });
        Ok(args)
    }
}

#[messages]
impl ProposalManager {
    #[message]
    pub async fn create_proposal(
        &mut self,
        kind: ProposalKind,
        initiator_id: i32,
        ttl_secs: Option<i64>,
    ) -> Result<i32, Error> {
        let ttl = ttl_secs.unwrap_or(DEFAULT_TTL_SECS);
        let expires_at = SqliteTimestamp::from(Utc::now() + chrono::Duration::seconds(ttl));

        let new_proposal = NewProposal {
            kind: kind.kind_str().to_owned(),
            payload: kind.encode_payload(),
            initiator_id,
            expires_at,
        };

        let mut conn = self.db.get().await?;
        let id: i32 = diesel::insert_into(schema::proposal::table)
            .values(&new_proposal)
            .returning(schema::proposal::id)
            .get_result(&mut conn)
            .await?;

        Ok(id)
    }

    #[message]
    pub async fn query_pending(&mut self, operator_id: i32) -> Vec<ProposalSummary> {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::as_conversions,
            reason = "fixme! #84; this will break in 2038"
        )]
        let now_ts = Utc::now().timestamp() as i32;

        let Ok(mut conn) = self.db.get().await else {
            warn!("query_pending: failed to acquire DB connection");
            return vec![];
        };

        let voted_ids: Vec<i32> = schema::proposal_vote::table
            .filter(schema::proposal_vote::operator_id.eq(operator_id))
            .select(schema::proposal_vote::proposal_id)
            .load(&mut conn)
            .await
            .unwrap_or_default();

        let proposals: Vec<Proposal> = schema::proposal::table
            .filter(schema::proposal::status.eq(ProposalStatus::Pending))
            .filter(schema::proposal::expires_at.gt(now_ts))
            .filter(diesel::dsl::not(schema::proposal::id.eq_any(&voted_ids)))
            .load(&mut conn)
            .await
            .unwrap_or_default();

        let mut summaries = Vec::with_capacity(proposals.len());
        for p in proposals {
            let approve_count: i64 = schema::proposal_vote::table
                .filter(schema::proposal_vote::proposal_id.eq(p.id))
                .filter(schema::proposal_vote::approve.eq(true))
                .count()
                .get_result(&mut conn)
                .await
                .unwrap_or(0);
            let reject_count: i64 = schema::proposal_vote::table
                .filter(schema::proposal_vote::proposal_id.eq(p.id))
                .filter(schema::proposal_vote::approve.eq(false))
                .count()
                .get_result(&mut conn)
                .await
                .unwrap_or(0);
            summaries.push(ProposalSummary {
                id: p.id,
                kind: p.kind,
                initiator_id: p.initiator_id,
                expires_at: p.expires_at,
                approve_count,
                reject_count,
            });
        }
        summaries
    }

    #[message]
    pub async fn expire_stale(&mut self) -> usize {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::as_conversions,
            reason = "fixme! #84; this will break in 2038"
        )]
        let now_ts = Utc::now().timestamp() as i32;

        let Ok(mut conn) = self.db.get().await else {
            warn!("expire_stale: failed to acquire DB connection");
            return 0;
        };

        diesel::update(schema::proposal::table)
            .filter(schema::proposal::status.eq(ProposalStatus::Pending))
            .filter(schema::proposal::expires_at.lt(now_ts))
            .set(schema::proposal::status.eq(ProposalStatus::Expired))
            .execute(&mut conn)
            .await
            .unwrap_or(0)
    }

    #[message]
    pub async fn cast_vote(
        &mut self,
        proposal_id: i32,
        operator_id: i32,
        approve: bool,
        signature: Vec<u8>,
    ) -> Result<VoteOutcome, Error> {
        use arbiter_crypto::authn::{self, GOVERNANCE_CONTEXT};

        let mut conn = self.db.get().await?;

        // Load proposal — must exist
        let proposal: Proposal = schema::proposal::table
            .find(proposal_id)
            .first(&mut conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::ProposalNotFound,
                other => Error::DatabaseQuery(other),
            })?;

        // Check for duplicate vote before status check so AlreadyVoted takes priority
        let existing: i64 = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::proposal_vote::operator_id.eq(operator_id))
            .count()
            .get_result(&mut conn)
            .await?;
        if existing > 0 {
            return Err(Error::AlreadyVoted);
        }

        if proposal.status != ProposalStatus::Pending {
            return Err(Error::ProposalNotPending);
        }

        // Load operator public key from operator_identity
        let pubkey_bytes: Vec<u8> = schema::operator_identity::table
            .find(operator_id)
            .select(schema::operator_identity::public_key)
            .first(&mut conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::OperatorNotFound,
                other => Error::DatabaseQuery(other),
            })?;

        let pubkey = authn::PublicKey::try_from(pubkey_bytes.as_slice())
            .map_err(|()| Error::InvalidSignature)?;

        // Canonical vote message: proposal_id (i64 big-endian) || approve (u8)
        let mut vote_msg = Vec::with_capacity(9);
        vote_msg.extend_from_slice(&i64::from(proposal_id).to_be_bytes());
        vote_msg.push(u8::from(approve));

        let auth_sig = authn::Signature::try_from(signature.as_slice())
            .map_err(|()| Error::InvalidSignature)?;

        if !pubkey.verify_message(&vote_msg, GOVERNANCE_CONTEXT, &auth_sig) {
            return Err(Error::InvalidSignature);
        }

        // Insert vote
        diesel::insert_into(schema::proposal_vote::table)
            .values(&NewProposalVote {
                proposal_id,
                operator_id,
                approve,
                signature,
            })
            .execute(&mut conn)
            .await?;

        // Quorum check
        let total_operators: i64 = schema::operator_identity::table
            .count()
            .get_result(&mut conn)
            .await?;
        let recovery_active = Self::is_recovery_active_conn(&mut conn).await?;
        let total_recovery: i64 = if recovery_active {
            schema::recovery_operator_identity::table
                .count()
                .get_result(&mut conn)
                .await?
        } else {
            0
        };
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::as_conversions,
            reason = "operator count is always a small positive integer"
        )]
        let threshold = if ProposalKind::requires_full_quorum(&proposal.kind) {
            // §3.3: key-rotation proposals require every eligible voter to approve
            // §3.5: when recovery is active, recovery operators also vote on replace_operator
            (total_operators + total_recovery) as usize
        } else {
            crate::crypto::shamir::shamir_threshold(total_operators as usize)
        };

        let ordinary_approve: i64 = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::proposal_vote::approve.eq(true))
            .count()
            .get_result(&mut conn)
            .await?;
        let recovery_approve: i64 = schema::recovery_proposal_vote::table
            .filter(schema::recovery_proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::recovery_proposal_vote::approve.eq(true))
            .count()
            .get_result(&mut conn)
            .await?;
        let approve_count = ordinary_approve + recovery_approve;

        let ordinary_reject: i64 = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::proposal_vote::approve.eq(false))
            .count()
            .get_result(&mut conn)
            .await?;
        let recovery_reject: i64 = schema::recovery_proposal_vote::table
            .filter(schema::recovery_proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::recovery_proposal_vote::approve.eq(false))
            .count()
            .get_result(&mut conn)
            .await?;
        let reject_count = ordinary_reject + recovery_reject;

        #[expect(
            clippy::cast_possible_wrap,
            clippy::as_conversions,
            reason = "threshold is derived from operator count, always fits i64"
        )]
        let threshold_i64 = threshold as i64;

        if approve_count >= threshold_i64 {
            diesel::update(schema::proposal::table.find(proposal_id))
                .set(schema::proposal::status.eq(ProposalStatus::Approved))
                .execute(&mut conn)
                .await?;
            drop(conn); // release connection before async execution
            self.execute_proposal(&proposal).await?;
            return Ok(VoteOutcome::QuorumApproved);
        }

        let total_eligible = total_operators + total_recovery;
        if reject_count > total_eligible - threshold_i64 {
            diesel::update(schema::proposal::table.find(proposal_id))
                .set(schema::proposal::status.eq(ProposalStatus::Rejected))
                .execute(&mut conn)
                .await?;
            return Ok(VoteOutcome::QuorumRejected);
        }

        Ok(VoteOutcome::Pending)
    }

    /// §3.6: Any ordinary operator may request recovery wake-up.
    /// Fails if a wake-up is already pending or active.
    #[message]
    pub async fn request_recovery_wakeup(&mut self, operator_id: i32) -> Result<(), Error> {
        let mut conn = self.db.get().await?;
        if Self::has_uncancelled_wakeup(&mut conn).await? {
            return Err(Error::WakeupAlreadyPending);
        }
        diesel::insert_into(schema::recovery_wakeup_request::table)
            .values(&NewRecoveryWakeupRequest {
                requested_by: operator_id,
            })
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    /// §3.6: Any ordinary operator may cancel a pending wake-up request.
    /// Fails if there is no uncancelled request.
    #[message]
    pub async fn cancel_recovery_wakeup(&mut self, operator_id: i32) -> Result<(), Error> {
        let mut conn = self.db.get().await?;
        let rows_updated = diesel::update(schema::recovery_wakeup_request::table)
            .filter(schema::recovery_wakeup_request::cancelled_at.is_null())
            .set((
                schema::recovery_wakeup_request::cancelled_by.eq(Some(operator_id)),
                schema::recovery_wakeup_request::cancelled_at.eq(Some(SqliteTimestamp::now())),
            ))
            .execute(&mut conn)
            .await?;
        if rows_updated == 0 {
            return Err(Error::NoActiveWakeup);
        }
        Ok(())
    }

    /// §3.5: Recovery operators may only vote on operator replacement proposals.
    /// §3.6: Voting is gated behind recovery being active (14-day window elapsed).
    #[message]
    pub async fn cast_recovery_vote(
        &mut self,
        proposal_id: i32,
        recovery_operator_id: i32,
        approve: bool,
        signature: Vec<u8>,
    ) -> Result<VoteOutcome, Error> {
        use arbiter_crypto::authn::{self, GOVERNANCE_CONTEXT};

        let mut conn = self.db.get().await?;

        let proposal: Proposal = schema::proposal::table
            .find(proposal_id)
            .first(&mut conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::ProposalNotFound,
                other => Error::DatabaseQuery(other),
            })?;

        if proposal.kind.parse::<ProposalKindTag>() != Ok(ProposalKindTag::ReplaceOperator) {
            return Err(Error::NotAllowedForRecoveryOperator);
        }

        if !Self::is_recovery_active_conn(&mut conn).await? {
            return Err(Error::RecoveryNotActive);
        }

        let existing: i64 = schema::recovery_proposal_vote::table
            .filter(schema::recovery_proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::recovery_proposal_vote::recovery_operator_id.eq(recovery_operator_id))
            .count()
            .get_result(&mut conn)
            .await?;
        if existing > 0 {
            return Err(Error::AlreadyVoted);
        }

        if proposal.status != ProposalStatus::Pending {
            return Err(Error::ProposalNotPending);
        }

        let pubkey_bytes: Vec<u8> = schema::recovery_operator_identity::table
            .find(recovery_operator_id)
            .select(schema::recovery_operator_identity::public_key)
            .first(&mut conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::OperatorNotFound,
                other => Error::DatabaseQuery(other),
            })?;

        let pubkey = authn::PublicKey::try_from(pubkey_bytes.as_slice())
            .map_err(|()| Error::InvalidSignature)?;

        let mut vote_msg = Vec::with_capacity(9);
        vote_msg.extend_from_slice(&i64::from(proposal_id).to_be_bytes());
        vote_msg.push(u8::from(approve));

        let auth_sig = authn::Signature::try_from(signature.as_slice())
            .map_err(|()| Error::InvalidSignature)?;

        if !pubkey.verify_message(&vote_msg, GOVERNANCE_CONTEXT, &auth_sig) {
            return Err(Error::InvalidSignature);
        }

        diesel::insert_into(schema::recovery_proposal_vote::table)
            .values(&NewRecoveryProposalVote {
                proposal_id,
                recovery_operator_id,
                approve,
                signature,
            })
            .execute(&mut conn)
            .await?;

        // Quorum: all ordinary + all recovery operators must approve (§3.3 + §3.5)
        let total_ordinary: i64 = schema::operator_identity::table
            .count()
            .get_result(&mut conn)
            .await?;
        let total_recovery: i64 = schema::recovery_operator_identity::table
            .count()
            .get_result(&mut conn)
            .await?;
        let threshold_i64 = total_ordinary + total_recovery;

        let ordinary_approve: i64 = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::proposal_vote::approve.eq(true))
            .count()
            .get_result(&mut conn)
            .await?;
        let recovery_approve: i64 = schema::recovery_proposal_vote::table
            .filter(schema::recovery_proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::recovery_proposal_vote::approve.eq(true))
            .count()
            .get_result(&mut conn)
            .await?;
        let approve_count = ordinary_approve + recovery_approve;

        if approve_count >= threshold_i64 {
            diesel::update(schema::proposal::table.find(proposal_id))
                .set(schema::proposal::status.eq(ProposalStatus::Approved))
                .execute(&mut conn)
                .await?;
            drop(conn);
            self.execute_proposal(&proposal).await?;
            return Ok(VoteOutcome::QuorumApproved);
        }

        let recovery_reject: i64 = schema::recovery_proposal_vote::table
            .filter(schema::recovery_proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::recovery_proposal_vote::approve.eq(false))
            .count()
            .get_result(&mut conn)
            .await?;
        let ordinary_reject: i64 = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::proposal_vote::approve.eq(false))
            .count()
            .get_result(&mut conn)
            .await?;
        let reject_count = ordinary_reject + recovery_reject;

        if reject_count > threshold_i64 - approve_count - reject_count {
            diesel::update(schema::proposal::table.find(proposal_id))
                .set(schema::proposal::status.eq(ProposalStatus::Rejected))
                .execute(&mut conn)
                .await?;
            return Ok(VoteOutcome::QuorumRejected);
        }

        Ok(VoteOutcome::Pending)
    }
}

impl ProposalManager {
    const WAKEUP_DELAY_SECS: i32 = 14 * 24 * 60 * 60;

    /// Returns true when an uncancelled wakeup request has passed the 14-day dispute window.
    async fn is_recovery_active_conn(conn: &mut db::DatabaseConnection) -> Result<bool, Error> {
        let count: i64 = schema::recovery_wakeup_request::table
            .filter(schema::recovery_wakeup_request::cancelled_at.is_null())
            .filter(
                schema::recovery_wakeup_request::requested_at.le(diesel::dsl::sql::<
                    diesel::sql_types::Integer,
                >(&format!(
                    "unixepoch('now') - {}",
                    Self::WAKEUP_DELAY_SECS
                ))),
            )
            .count()
            .get_result(conn)
            .await?;
        Ok(count > 0)
    }

    /// Returns true when there is any uncancelled wakeup request (pending or active).
    async fn has_uncancelled_wakeup(conn: &mut db::DatabaseConnection) -> Result<bool, Error> {
        let count: i64 = schema::recovery_wakeup_request::table
            .filter(schema::recovery_wakeup_request::cancelled_at.is_null())
            .count()
            .get_result(conn)
            .await?;
        Ok(count > 0)
    }

    async fn execute_proposal(&self, proposal: &Proposal) -> Result<(), Error> {
        let kind = ProposalKind::decode(&proposal.kind, &proposal.payload)
            .map_err(Error::ExecutionFailed)?;
        match kind {
            ProposalKind::ApproveSdkClient { client_id } => {
                self.execute_approve_sdk_client(client_id).await
            }
            ProposalKind::GrantWalletAccess {
                wallet_id,
                client_id,
            } => self.execute_grant_wallet_access(wallet_id, client_id).await,
            ProposalKind::ApproveServerUpdate => Ok(()),
            ProposalKind::ReplaceOperator {
                old_operator_id,
                new_pubkey,
            } => {
                self.execute_replace_operator(old_operator_id, new_pubkey)
                    .await
            }
            ProposalKind::UpdateShamirParameters { new_n } => {
                self.execute_update_shamir_parameters(new_n).await
            }
            ProposalKind::ApprovePersistentGrant { payload_bytes } => {
                self.execute_approve_persistent_grant(payload_bytes).await
            }
            ProposalKind::ApproveOneOffTransaction { payload_bytes } => {
                self.execute_approve_one_off_transaction(proposal.id, payload_bytes)
                    .await
            }
        }
    }

    async fn execute_grant_wallet_access(
        &self,
        wallet_id: i32,
        client_id: i32,
    ) -> Result<(), Error> {
        use crate::db::models::EvmWalletId;

        let mut conn = self.db.get().await.map_err(Error::DatabaseConnection)?;

        diesel::insert_into(schema::evm_wallet_access::table)
            .values((
                schema::evm_wallet_access::wallet_id.eq(EvmWalletId::from_raw(wallet_id)),
                schema::evm_wallet_access::client_id.eq(client_id),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| Error::ExecutionFailed(format!("grant wallet access: {e}")))?;

        Ok(())
    }

    /// Updates the old operator's public key in-place (preserving their DB id and history),
    /// removes their old Shamir share, then begins a coordinated re-key (§3.3).
    async fn execute_replace_operator(
        &self,
        old_operator_id: i32,
        new_pubkey: Vec<u8>,
    ) -> Result<(), Error> {
        let mut conn = self.db.get().await.map_err(Error::DatabaseConnection)?;

        diesel::update(schema::operator_identity::table)
            .filter(schema::operator_identity::id.eq(old_operator_id))
            .set(schema::operator_identity::public_key.eq(&new_pubkey))
            .execute(&mut conn)
            .await
            .map_err(|e| Error::ExecutionFailed(format!("update operator pubkey: {e}")))?;

        // Remove the old Shamir share; finalize_rekey will store a fresh one.
        diesel::delete(schema::operator::table)
            .filter(schema::operator::id.eq(Some(old_operator_id)))
            .execute(&mut conn)
            .await
            .map_err(|e| Error::ExecutionFailed(format!("remove old operator share: {e}")))?;

        drop(conn);

        self.vault_coordinator
            .ask(StartRekey {})
            .await
            .map_err(|e| Error::ExecutionFailed(format!("start rekey: {e}")))?;

        Ok(())
    }

    /// Triggers a Shamir re-key with the current operator set (§3.3).
    async fn execute_update_shamir_parameters(&self, _new_n: u8) -> Result<(), Error> {
        self.vault_coordinator
            .ask(StartRekey {})
            .await
            .map_err(|e| Error::ExecutionFailed(format!("start rekey: {e}")))?;
        Ok(())
    }

    async fn execute_approve_one_off_transaction(
        &self,
        proposal_id: i32,
        payload_bytes: Vec<u8>,
    ) -> Result<(), Error> {
        use crate::actors::evm::ClientSignTransaction;
        use crate::db::models::NewProposalResult;
        use alloy::{
            consensus::TxEip1559,
            eips::eip2930::AccessList,
            primitives::{Address, Bytes, TxKind, U256},
        };
        use arbiter_proto::proto::operator::governance::ApproveOneOffTransactionPayload;
        use prost::Message as _;

        let p = ApproveOneOffTransactionPayload::decode(payload_bytes.as_slice())
            .map_err(|e| Error::ExecutionFailed(format!("decode one-off tx payload: {e}")))?;

        let wallet_address = Address::from_slice(p.wallet_address.as_slice());
        let to = Address::from_slice(p.to.as_slice());

        let transaction = TxEip1559 {
            chain_id: p.chain_id,
            nonce: p.nonce,
            gas_limit: p.gas_limit,
            max_fee_per_gas: u128::from_be_bytes(
                p.max_fee_per_gas
                    .as_slice()
                    .try_into()
                    .map_err(|_| Error::ExecutionFailed("invalid max_fee_per_gas".to_owned()))?,
            ),
            max_priority_fee_per_gas: u128::from_be_bytes(
                p.max_priority_fee_per_gas
                    .as_slice()
                    .try_into()
                    .map_err(|_| {
                        Error::ExecutionFailed("invalid max_priority_fee_per_gas".to_owned())
                    })?,
            ),
            to: TxKind::Call(to),
            value: U256::from_be_slice(p.value.as_slice()),
            input: Bytes::from(p.input),
            access_list: AccessList::default(),
        };

        let sig = self
            .evm
            .ask(ClientSignTransaction {
                client_id: p.client_id,
                wallet_address,
                transaction,
            })
            .await
            .map_err(|e| Error::ExecutionFailed(format!("sign one-off tx: {e}")))?;

        let mut conn = self.db.get().await.map_err(Error::DatabaseConnection)?;
        diesel::insert_into(schema::proposal_result::table)
            .values(NewProposalResult {
                proposal_id,
                data: sig.as_bytes().to_vec(),
            })
            .execute(&mut conn)
            .await
            .map_err(|e| Error::ExecutionFailed(format!("store proposal result: {e}")))?;

        Ok(())
    }

    async fn execute_approve_persistent_grant(&self, payload_bytes: Vec<u8>) -> Result<(), Error> {
        use crate::{
            actors::evm::OperatorCreateGrant,
            evm::policies::{
                SharedGrantSettings, SpecificGrant, TransactionRateLimit, VolumeRateLimit,
                ether_transfer, token_transfers,
            },
        };
        use alloy::primitives::{Address, U256};
        use arbiter_proto::proto::operator::governance::{
            ApprovePersistentGrantPayload, approve_persistent_grant_payload::Specific,
        };
        use chrono::Duration;
        use prost::Message as _;

        let payload = ApprovePersistentGrantPayload::decode(payload_bytes.as_slice())
            .map_err(|e| Error::ExecutionFailed(format!("decode grant payload: {e}")))?;

        let basic = SharedGrantSettings {
            wallet_access_id: payload.wallet_access_id,
            chain: payload.chain_id,
            valid_from: payload
                .valid_from_secs
                .and_then(|s| chrono::DateTime::from_timestamp(s, 0)),
            valid_until: payload
                .valid_until_secs
                .and_then(|s| chrono::DateTime::from_timestamp(s, 0)),
            max_gas_fee_per_gas: payload
                .max_gas_fee_per_gas
                .map(|b| U256::from_be_slice(b.as_slice())),
            max_priority_fee_per_gas: payload
                .max_priority_fee_per_gas
                .map(|b| U256::from_be_slice(b.as_slice())),
            rate_limit: payload.rate_limit.map(|r| TransactionRateLimit {
                count: r.count,
                window: Duration::seconds(r.window_secs),
            }),
        };

        let grant = match payload.specific {
            Some(Specific::EtherTransfer(spec)) => {
                let target: Vec<Address> = spec
                    .targets
                    .iter()
                    .map(|b| Address::from_slice(b.as_slice()))
                    .collect();
                let limit = spec
                    .limit
                    .map(|l| VolumeRateLimit {
                        max_volume: U256::from_be_slice(l.max_volume.as_slice()),
                        window: Duration::seconds(l.window_secs),
                    })
                    .ok_or_else(|| {
                        Error::ExecutionFailed("missing ether transfer limit".to_owned())
                    })?;
                SpecificGrant::EtherTransfer(ether_transfer::Settings { target, limit })
            }
            Some(Specific::TokenTransfer(spec)) => {
                let token_contract = Address::from_slice(spec.token_contract.as_slice());
                let target = spec.target.map(|b| Address::from_slice(b.as_slice()));
                let volume_limits: Vec<VolumeRateLimit> = spec
                    .volume_limits
                    .iter()
                    .map(|l| VolumeRateLimit {
                        max_volume: U256::from_be_slice(l.max_volume.as_slice()),
                        window: Duration::seconds(l.window_secs),
                    })
                    .collect();
                SpecificGrant::TokenTransfer(token_transfers::Settings {
                    token_contract,
                    target,
                    volume_limits,
                })
            }
            None => return Err(Error::ExecutionFailed("missing grant specific".to_owned())),
        };

        self.evm
            .ask(OperatorCreateGrant { basic, grant })
            .await
            .map_err(|e| Error::ExecutionFailed(format!("create grant: {e}")))?;

        Ok(())
    }

    async fn execute_approve_sdk_client(&self, client_id: i32) -> Result<(), Error> {
        use crate::{crypto::integrity, peers::client::ClientCredentials};
        use arbiter_crypto::authn;

        let mut conn = self.db.get().await.map_err(Error::DatabaseConnection)?;

        let pubkey_bytes: Vec<u8> = schema::program_client::table
            .find(client_id)
            .select(schema::program_client::public_key)
            .first(&mut conn)
            .await
            .map_err(|e| Error::ExecutionFailed(format!("client not found: {e}")))?;

        let pubkey = authn::PublicKey::try_from(pubkey_bytes.as_slice())
            .map_err(|()| Error::ExecutionFailed("invalid client public key".to_owned()))?;

        let creds = ClientCredentials { pubkey };

        integrity::sign_entity(&mut conn, &self.vault, &creds, client_id)
            .await
            .map_err(|e| {
                error!(?e, "Failed to sign integrity envelope for client");
                Error::ExecutionFailed(e.to_string())
            })
    }
}
