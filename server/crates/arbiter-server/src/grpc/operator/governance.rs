use crate::{
    actors::proposal_manager::{Error as ProposalError, ProposalKind, VoteOutcome},
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
        Some(ProtoKind::ApproveSdkClient(p)) => ProposalKind::ApproveSdkClient {
            client_id: p.client_id,
        },
        Some(ProtoKind::GrantWalletAccess(p)) => ProposalKind::GrantWalletAccess {
            wallet_id: p.wallet_id,
            client_id: p.client_id,
        },
        Some(ProtoKind::ApproveServerUpdate(_)) => ProposalKind::ApproveServerUpdate,
        Some(ProtoKind::ReplaceOperator(p)) => ProposalKind::ReplaceOperator {
            new_pubkey: p.new_pubkey,
        },
        Some(ProtoKind::UpdateShamirParameters(p)) => ProposalKind::UpdateShamirParameters {
            #[expect(clippy::cast_possible_truncation, clippy::as_conversions, reason = "new_n is always a small operator count")]
            new_n: p.new_n as u8,
        },
        None => return Err(Status::invalid_argument("Missing proposal kind")),
    };
    let ttl_secs = req.ttl_secs.map(i64::from);

    let proposal_id = actor
        .ask(HandleCreateProposal { kind, ttl_secs })
        .await
        .map_err(|e| {
            warn!(?e, "create_proposal failed");
            Status::internal("Failed to create proposal")
        })?;

    Ok(Some(wrap(GovResponsePayload::Created(
        proto_gov::CreateProposalResponse { proposal_id },
    ))))
}

async fn handle_vote(
    actor: &ActorRef<OperatorSession>,
    req: proto_gov::CastVoteRequest,
) -> Result<Option<OperatorResponsePayload>, Status> {
    let result = actor
        .ask(HandleCastVote {
            proposal_id: req.proposal_id,
            approve: req.approve,
            signature: req.signature,
        })
        .await;

    let outcome = match result {
        Ok(VoteOutcome::Pending) => ProtoVoteOutcome::Pending,
        Ok(VoteOutcome::QuorumApproved) => ProtoVoteOutcome::Approved,
        Ok(VoteOutcome::QuorumRejected) => ProtoVoteOutcome::Rejected,
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
            id: s.id,
            kind: s.kind,
            initiator_id: s.initiator_id,
            expires_at: s.expires_at.0.timestamp(),
            approve_count: s.approve_count,
            reject_count: s.reject_count,
        })
        .collect();

    Ok(Some(wrap(GovResponsePayload::Pending(
        QueryPendingResponse { proposals },
    ))))
}
