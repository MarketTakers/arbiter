use crate::{
    actors::proposal_manager::{Error as ProposalError, VoteOutcome},
    db::models::{OperatorIdentityId, ProposalId},
    db::proposal::{
        ProposalKind, approve_sdk_client, grant_wallet_access, one_off_transaction,
        persistent_grant, replace_operator,
    },
    peers::operator::{
        OperatorSession,
        session::handlers::{HandleCastVote, HandleCreateProposal, HandleQueryPending},
    },
};
use arbiter_proto::proto::operator::{
    governance::{
        self as proto_gov, CreateProposalRequest, QueryPendingRequest, QueryPendingResponse,
        VoteOutcome as ProtoVoteOutcome, create_proposal_request::Kind as ProtoKind,
        request::Payload as GovRequestPayload, response::Payload as GovResponsePayload,
    },
    operator_response::Payload as OperatorResponsePayload,
};
use kameo::actor::ActorRef;
use tonic::Status;
use tracing::warn;

const fn wrap(payload: GovResponsePayload) -> OperatorResponsePayload {
    OperatorResponsePayload::Governance(proto_gov::Response {
        payload: Some(payload),
    })
}

pub(super) async fn dispatch(
    actor: &ActorRef<OperatorSession>,
    req: proto_gov::Request,
) -> Result<Option<OperatorResponsePayload>, Status> {
    let Some(payload) = req.payload else {
        return Err(Status::invalid_argument(
            "Missing governance request payload",
        ));
    };

    match payload {
        GovRequestPayload::Create(req) => handle_create(actor, req).await,
        GovRequestPayload::Vote(req) => handle_vote(actor, req).await,
        GovRequestPayload::Query(QueryPendingRequest {}) => handle_query(actor).await,
    }
}

async fn handle_create(
    actor: &ActorRef<OperatorSession>,
    req: CreateProposalRequest,
) -> Result<Option<OperatorResponsePayload>, Status> {
    let kind = match req.kind {
        Some(ProtoKind::ApproveSdkClient(p)) => {
            ProposalKind::ApproveSdkClient(approve_sdk_client::Settings {
                client_id: p.client_id,
            })
        }
        Some(ProtoKind::GrantWalletAccess(p)) => {
            ProposalKind::GrantWalletAccess(grant_wallet_access::Settings {
                wallet_id: p.wallet_id,
                client_id: p.client_id,
            })
        }
        Some(ProtoKind::ReplaceOperator(p)) => {
            ProposalKind::ReplaceOperator(replace_operator::Settings {
                old_operator_id: OperatorIdentityId::from_raw(p.old_operator_id),
                new_pubkey: p.new_pubkey,
            })
        }
        Some(ProtoKind::TriggerRekey(())) => ProposalKind::TriggerRekey,
        Some(ProtoKind::ApprovePersistentGrant(p)) => {
            ProposalKind::ApprovePersistentGrant(Box::new(parse_persistent_grant(p)?))
        }
        Some(ProtoKind::ApproveOneOffTransaction(p)) => {
            ProposalKind::ApproveOneOffTransaction(Box::new(parse_one_off_transaction(p)?))
        }
        None => return Err(Status::invalid_argument("Missing proposal kind")),
    };
    let proposal_id = actor
        .ask(HandleCreateProposal {
            kind,
            ttl_secs: req.ttl_secs,
        })
        .await
        .map_err(|e| {
            warn!(?e, "create_proposal failed");
            Status::internal("Failed to create proposal")
        })?;

    Ok(Some(wrap(GovResponsePayload::Created(
        proto_gov::CreateProposalResponse {
            proposal_id: proposal_id.to_raw(),
        },
    ))))
}

/// Validates the grant where the request enters, so a malformed one is refused before
/// any operator votes on it instead of failing after quorum.
fn parse_persistent_grant(
    p: proto_gov::ApprovePersistentGrantPayload,
) -> Result<persistent_grant::Settings, Status> {
    use proto_gov::approve_persistent_grant_payload::Specific;

    let volume =
        |l: proto_gov::VolumeLimitProto| -> Result<persistent_grant::VolumeLimit, Status> {
            Ok(persistent_grant::VolumeLimit {
                max_volume: fixed(&l.max_volume, "max_volume must be 32 bytes")?,
                window_secs: l.window_secs,
            })
        };

    let specific = match p.specific {
        Some(Specific::EtherTransfer(spec)) => {
            let targets = spec
                .targets
                .iter()
                .map(|target| fixed(target, "ether transfer target must be 20 bytes"))
                .collect::<Result<Vec<_>, _>>()?;
            let limit = spec
                .limit
                .ok_or_else(|| Status::invalid_argument("missing ether transfer limit"))?;
            persistent_grant::Specific::EtherTransfer {
                targets,
                limit: volume(limit)?,
            }
        }
        Some(Specific::TokenTransfer(spec)) => {
            let volume_limits = spec
                .volume_limits
                .into_iter()
                .map(volume)
                .collect::<Result<Vec<_>, _>>()?;
            persistent_grant::Specific::TokenTransfer {
                token_contract: fixed(&spec.token_contract, "token_contract must be 20 bytes")?,
                receiver: spec
                    .target
                    .map(|t| fixed(&t, "token transfer target must be 20 bytes"))
                    .transpose()?,
                volume_limits,
            }
        }
        None => return Err(Status::invalid_argument("missing grant specific")),
    };

    Ok(persistent_grant::Settings {
        wallet_access_id: p.wallet_access_id,
        chain_id: p.chain_id,
        valid_from_secs: p.valid_from_secs,
        valid_until_secs: p.valid_until_secs,
        max_gas_fee_per_gas: p
            .max_gas_fee_per_gas
            .map(|v| fixed(&v, "max_gas_fee_per_gas must be 32 bytes"))
            .transpose()?,
        max_priority_fee_per_gas: p
            .max_priority_fee_per_gas
            .map(|v| fixed(&v, "max_priority_fee_per_gas must be 32 bytes"))
            .transpose()?,
        rate_limit: p.rate_limit.map(|r| persistent_grant::RateLimit {
            count: r.count,
            window_secs: r.window_secs,
        }),
        specific,
    })
}

fn fixed<const N: usize>(bytes: &[u8], message: &'static str) -> Result<[u8; N], Status> {
    <[u8; N]>::try_from(bytes).map_err(|_| Status::invalid_argument(message))
}

/// Validates the transaction where the request enters, so a malformed one is refused
/// before any operator votes on it instead of failing after quorum.
fn parse_one_off_transaction(
    p: proto_gov::ApproveOneOffTransactionPayload,
) -> Result<one_off_transaction::Settings, Status> {
    Ok(one_off_transaction::Settings {
        client_id: p.client_id,
        wallet_address: fixed(&p.wallet_address, "wallet_address must be 20 bytes")?,
        chain_id: p.chain_id,
        nonce: p.nonce,
        gas_limit: p.gas_limit,
        max_fee_per_gas: u128::from_be_bytes(fixed(
            &p.max_fee_per_gas,
            "max_fee_per_gas must be 16 bytes",
        )?),
        max_priority_fee_per_gas: u128::from_be_bytes(fixed(
            &p.max_priority_fee_per_gas,
            "max_priority_fee_per_gas must be 16 bytes",
        )?),
        to: fixed(&p.to, "to must be 20 bytes")?,
        value: fixed(&p.value, "value must be 32 bytes")?,
        input: p.input,
    })
}

async fn handle_vote(
    actor: &ActorRef<OperatorSession>,
    req: proto_gov::CastVoteRequest,
) -> Result<Option<OperatorResponsePayload>, Status> {
    let result = actor
        .ask(HandleCastVote {
            proposal_id: ProposalId::from_raw(req.proposal_id),
            approve: req.approve,
            signature: req.signature,
        })
        .await;

    let outcome = match result {
        Ok(VoteOutcome::Pending) => ProtoVoteOutcome::Pending,
        Ok(VoteOutcome::Approved) => ProtoVoteOutcome::Approved,
        Ok(VoteOutcome::Rejected) => ProtoVoteOutcome::Rejected,
        Err(kameo::error::SendError::HandlerError(ProposalError::AlreadyVoted)) => {
            return Err(Status::invalid_argument("Already voted on this proposal"));
        }
        Err(kameo::error::SendError::HandlerError(ProposalError::InvalidSignature)) => {
            return Err(Status::invalid_argument("Invalid vote signature"));
        }
        Err(kameo::error::SendError::HandlerError(ProposalError::ProposalNotFound)) => {
            return Err(Status::not_found("Proposal not found"));
        }
        Err(e) => {
            warn!(?e, "cast_vote failed");
            return Err(Status::internal("Failed to cast vote"));
        }
    };

    Ok(Some(wrap(GovResponsePayload::Voted(
        proto_gov::VoteResponse {
            outcome: outcome.into(),
        },
    ))))
}

async fn handle_query(
    actor: &ActorRef<OperatorSession>,
) -> Result<Option<OperatorResponsePayload>, Status> {
    let summaries = actor.ask(HandleQueryPending {}).await.unwrap_or_default();

    let proposals = summaries
        .into_iter()
        .map(|s| proto_gov::ProposalSummary {
            id: s.id.to_raw(),
            kind: <&'static str>::from(s.kind).to_owned(),
            initiator_id: s.initiator_id.to_raw(),
            expires_at: s.expires_at.0.timestamp(),
            approve_count: s.approve_count,
            reject_count: s.reject_count,
        })
        .collect();

    Ok(Some(wrap(GovResponsePayload::Pending(
        QueryPendingResponse { proposals },
    ))))
}
