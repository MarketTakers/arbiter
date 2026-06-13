use crate::{
    actors::vault::Vault,
    db::{
        self,
        models::{NewProposal, NewProposalVote, Proposal, ProposalStatus, SqliteTimestamp},
        schema,
    },
};
use chrono::Utc;
use diesel::{ExpressionMethods as _, QueryDsl};
use diesel_async::RunQueryDsl;
use kameo::{actor::ActorRef, messages};
use tracing::{error, warn};

pub const DEFAULT_TTL_SECS: i64 = 7 * 24 * 60 * 60; // 7 days

#[derive(Debug, Clone)]
pub enum ProposalKind {
    ApproveSdkClient { client_id: i32 },
    GrantWalletAccess { wallet_id: i32, client_id: i32 },
    ApproveServerUpdate,
}

impl ProposalKind {
    pub const fn kind_str(&self) -> &'static str {
        match self {
            Self::ApproveSdkClient { .. } => "approve_sdk_client",
            Self::GrantWalletAccess { .. } => "grant_wallet_access",
            Self::ApproveServerUpdate => "approve_server_update",
        }
    }

    pub fn encode_payload(&self) -> Vec<u8> {
        match self {
            Self::ApproveSdkClient { client_id } => client_id.to_be_bytes().to_vec(),
            Self::GrantWalletAccess { wallet_id, client_id } => {
                let mut buf = Vec::with_capacity(8);
                buf.extend_from_slice(&wallet_id.to_be_bytes());
                buf.extend_from_slice(&client_id.to_be_bytes());
                buf
            }
            Self::ApproveServerUpdate => vec![],
        }
    }

    pub fn decode(kind: &str, payload: &[u8]) -> Result<Self, String> {
        match kind {
            "approve_sdk_client" => {
                let bytes = <[u8; 4]>::try_from(payload)
                    .map_err(|_| "invalid payload for approve_sdk_client".to_owned())?;
                Ok(Self::ApproveSdkClient {
                    client_id: i32::from_be_bytes(bytes),
                })
            }
            "grant_wallet_access" => {
                let bytes = <[u8; 8]>::try_from(payload)
                    .map_err(|_| "invalid payload for grant_wallet_access".to_owned())?;
                Ok(Self::GrantWalletAccess {
                    wallet_id: i32::from_be_bytes(bytes[..4].try_into().unwrap()),
                    client_id: i32::from_be_bytes(bytes[4..].try_into().unwrap()),
                })
            }
            "approve_server_update" => Ok(Self::ApproveServerUpdate),
            other => Err(format!("unknown proposal kind: {other}")),
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
}

impl ProposalManager {
    pub const fn new(db: db::DatabasePool, vault: ActorRef<Vault>) -> Self {
        Self { db, vault }
    }
}

impl kameo::Actor for ProposalManager {
    type Args = Self;
    type Error = ();

    async fn on_start(
        args: Self::Args,
        actor_ref: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
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
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::as_conversions,
            reason = "operator count is always a small positive integer"
        )]
        let threshold = crate::crypto::shamir::shamir_threshold(total_operators as usize);

        let approve_count: i64 = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::proposal_vote::approve.eq(true))
            .count()
            .get_result(&mut conn)
            .await?;

        let reject_count: i64 = schema::proposal_vote::table
            .filter(schema::proposal_vote::proposal_id.eq(proposal_id))
            .filter(schema::proposal_vote::approve.eq(false))
            .count()
            .get_result(&mut conn)
            .await?;

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

        if reject_count > total_operators - threshold_i64 {
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
    async fn execute_proposal(&self, proposal: &Proposal) -> Result<(), Error> {
        let kind = ProposalKind::decode(&proposal.kind, &proposal.payload)
            .map_err(Error::ExecutionFailed)?;
        match kind {
            ProposalKind::ApproveSdkClient { client_id } => {
                self.execute_approve_sdk_client(client_id).await
            }
            ProposalKind::GrantWalletAccess { wallet_id, client_id } => {
                self.execute_grant_wallet_access(wallet_id, client_id).await
            }
            ProposalKind::ApproveServerUpdate => Ok(()),
        }
    }

    async fn execute_grant_wallet_access(&self, wallet_id: i32, client_id: i32) -> Result<(), Error> {
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

    async fn execute_approve_sdk_client(&self, client_id: i32) -> Result<(), Error> {
        use arbiter_crypto::authn;
        use crate::{
            crypto::integrity,
            peers::client::ClientCredentials,
        };

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
