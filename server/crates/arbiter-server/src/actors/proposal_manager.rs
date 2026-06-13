use crate::{
    actors::{evm::EvmActor, vault::Vault},
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
    ReplaceOperator { new_pubkey: Vec<u8> },
    UpdateShamirParameters { new_n: u8 },
    ApprovePersistentGrant { payload_bytes: Vec<u8> },
    ApproveOneOffTransaction { payload_bytes: Vec<u8> },
}

impl ProposalKind {
    pub const fn kind_str(&self) -> &'static str {
        match self {
            Self::ApproveSdkClient { .. } => "approve_sdk_client",
            Self::GrantWalletAccess { .. } => "grant_wallet_access",
            Self::ApproveServerUpdate => "approve_server_update",
            Self::ReplaceOperator { .. } => "replace_operator",
            Self::UpdateShamirParameters { .. } => "update_shamir_parameters",
            Self::ApprovePersistentGrant { .. } => "approve_persistent_grant",
            Self::ApproveOneOffTransaction { .. } => "approve_one_off_transaction",
        }
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
            Self::ReplaceOperator { new_pubkey } => {
                let len = u32::try_from(new_pubkey.len()).expect("pubkey len fits in u32");
                let mut buf = Vec::with_capacity(4 + new_pubkey.len());
                buf.extend_from_slice(&len.to_be_bytes());
                buf.extend_from_slice(new_pubkey);
                buf
            }
            Self::UpdateShamirParameters { new_n } => vec![*new_n],
            Self::ApprovePersistentGrant { payload_bytes } => payload_bytes.clone(),
            Self::ApproveOneOffTransaction { payload_bytes } => payload_bytes.clone(),
        }
    }

    /// Key-rotation proposals require every operator to approve (§3.3).
    #[must_use]
    pub fn requires_full_quorum(kind: &str) -> bool {
        matches!(kind, "replace_operator" | "update_shamir_parameters")
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
            "replace_operator" => {
                let (len_bytes, rest) = payload
                    .split_first_chunk::<4>()
                    .ok_or_else(|| "replace_operator payload too short".to_owned())?;
                let len = u32::from_be_bytes(*len_bytes);
                let len = usize::try_from(len).unwrap_or(usize::MAX);
                let new_pubkey = rest
                    .get(..len)
                    .ok_or_else(|| "replace_operator payload truncated".to_owned())?
                    .to_vec();
                Ok(Self::ReplaceOperator { new_pubkey })
            }
            "update_shamir_parameters" => {
                let &[new_n] = payload else {
                    return Err("invalid payload for update_shamir_parameters".to_owned());
                };
                Ok(Self::UpdateShamirParameters { new_n })
            }
            "approve_persistent_grant" => Ok(Self::ApprovePersistentGrant {
                payload_bytes: payload.to_vec(),
            }),
            "approve_one_off_transaction" => Ok(Self::ApproveOneOffTransaction {
                payload_bytes: payload.to_vec(),
            }),
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
    pub(crate) evm: ActorRef<EvmActor>,
}

impl ProposalManager {
    pub const fn new(
        db: db::DatabasePool,
        vault: ActorRef<Vault>,
        evm: ActorRef<EvmActor>,
    ) -> Self {
        Self { db, vault, evm }
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
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::as_conversions,
            reason = "operator count is always a small positive integer"
        )]
        let threshold = if ProposalKind::requires_full_quorum(&proposal.kind) {
            // §3.3: key-rotation proposals require every operator to approve
            total_operators as usize
        } else {
            crate::crypto::shamir::shamir_threshold(total_operators as usize)
        };

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
            ProposalKind::GrantWalletAccess {
                wallet_id,
                client_id,
            } => self.execute_grant_wallet_access(wallet_id, client_id).await,
            ProposalKind::ApproveServerUpdate => Ok(()),
            ProposalKind::ReplaceOperator { new_pubkey } => {
                self.execute_replace_operator(new_pubkey).await
            }
            ProposalKind::UpdateShamirParameters { new_n } => {
                self.execute_update_shamir_parameters(new_n)
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

    async fn execute_replace_operator(&self, new_pubkey: Vec<u8>) -> Result<(), Error> {
        let mut conn = self.db.get().await.map_err(Error::DatabaseConnection)?;
        diesel::insert_into(schema::operator_identity::table)
            .values(schema::operator_identity::public_key.eq(&new_pubkey))
            .execute(&mut conn)
            .await
            .map_err(|e| Error::ExecutionFailed(format!("replace operator: {e}")))?;
        Ok(())
    }

    #[expect(
        clippy::unused_self,
        clippy::unnecessary_wraps,
        reason = "signature must match other execute_* methods"
    )]
    fn execute_update_shamir_parameters(&self, new_n: u8) -> Result<(), Error> {
        warn!(
            new_n,
            "UpdateShamirParameters approved; Shamir re-keying must be performed out-of-band"
        );
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
