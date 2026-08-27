//! Canonical encoding and verification of governance vote signatures (§3.3).

use crate::db::models::ProposalId;
use arbiter_crypto::authn::{self, SigningContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum VerifyError {
    #[error("Malformed operator public key")]
    PublicKey,
    #[error("Malformed vote signature")]
    Signature,
    #[error("Signature does not match this vote")]
    Mismatch,
}

/// Canonical bytes an operator signs when voting: `proposal_id` as i64 big-endian,
/// followed by the approve flag as one byte.
///
/// The flag is part of the message on purpose: without it an approval could be
/// replayed as a rejection of the same proposal.
#[must_use]
pub fn vote_message(proposal_id: ProposalId, approve: bool) -> Vec<u8> {
    let mut message = Vec::with_capacity(9);
    message.extend_from_slice(&i64::from(proposal_id.to_raw()).to_be_bytes());
    message.push(u8::from(approve));
    message
}

/// Verifies a vote signature against an operator's stored public key.
pub fn verify_vote(
    public_key: &[u8],
    proposal_id: ProposalId,
    approve: bool,
    signature: &[u8],
) -> Result<(), VerifyError> {
    let public_key = authn::PublicKey::try_from(public_key).map_err(|()| VerifyError::PublicKey)?;
    let signature = authn::Signature::try_from(signature).map_err(|()| VerifyError::Signature)?;

    if public_key.verify_message(
        &vote_message(proposal_id, approve),
        SigningContext::GovernanceVote,
        &signature,
    ) {
        Ok(())
    } else {
        Err(VerifyError::Mismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::{VerifyError, verify_vote, vote_message};
    use crate::db::models::ProposalId;
    use arbiter_crypto::authn::{SigningContext, SigningKey};

    #[test]
    fn vote_message_is_the_id_then_the_approve_flag() {
        let message = vote_message(ProposalId::from_raw(0x0102), true);
        assert_eq!(message, vec![0, 0, 0, 0, 0, 0, 1, 2, 1]);
    }

    #[test]
    fn verify_vote_accepts_a_matching_signature() {
        let key = SigningKey::generate();
        let id = ProposalId::from_raw(42);
        let signature = key
            .sign_message(&vote_message(id, true), SigningContext::GovernanceVote)
            .unwrap();

        verify_vote(
            &key.public_key().to_bytes(),
            id,
            true,
            &signature.to_bytes(),
        )
        .expect("a signature over this exact vote must verify");
    }

    /// The decisive one: an approval must not verify as a rejection of the same
    /// proposal, or a captured vote could be replayed with its meaning flipped.
    #[test]
    fn verify_vote_rejects_a_flipped_approve_flag() {
        let key = SigningKey::generate();
        let id = ProposalId::from_raw(42);
        let signature = key
            .sign_message(&vote_message(id, true), SigningContext::GovernanceVote)
            .unwrap();

        assert!(matches!(
            verify_vote(
                &key.public_key().to_bytes(),
                id,
                false,
                &signature.to_bytes()
            ),
            Err(VerifyError::Mismatch)
        ));
    }
}
