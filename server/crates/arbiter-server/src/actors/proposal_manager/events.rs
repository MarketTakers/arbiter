use crate::db::{models::ProposalId, proposal::ProposalKind};

/// Published once a proposal reaches its approval threshold.
///
/// Executors subscribe on the global `MessageBus` and act on the kinds they own;
/// `ProposalManager` does not know who acts on an outcome, or whether anyone does.
#[derive(Debug, Clone)]
pub struct ProposalApproved {
    pub id: ProposalId,
    pub kind: ProposalKind,
}
