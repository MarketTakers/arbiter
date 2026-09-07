use vsss_rs::Gf256;

#[derive(Debug, thiserror::Error)]
pub enum ShamirError {
    #[error("Failed to split key: {0}")]
    Split(String),
    #[error("Failed to combine shares: {0}")]
    Combine(String),
}

/// Split `key` into `total` shares where any `threshold` shares can reconstruct it.
/// Each returned Vec<u8> is a share with format [`identifier_byte`, `value_bytes`...].
pub fn split_key(
    threshold: usize,
    total: usize,
    key: &[u8; 32],
    rng: impl rand_core::RngCore + rand_core::CryptoRng,
) -> Result<Vec<Vec<u8>>, ShamirError> {
    Gf256::split_array(threshold, total, key.as_slice(), rng)
        .map_err(|e| ShamirError::Split(format!("{e:?}")))
}

/// Returns the minimum number of shares required to reconstruct the secret
/// for a committee of `n` operators, or `None` for an empty committee.
#[must_use]
pub const fn shamir_threshold(n: usize) -> Option<usize> {
    match n {
        0 => None,
        1 => Some(1),
        2 => Some(2),
        n => Some(n / 2 + 1),
    }
}

/// Reconstruct the secret from `threshold` or more shares.
pub fn combine_shares(shares: &[Vec<u8>]) -> Result<[u8; 32], ShamirError> {
    let bytes = Gf256::combine_array(shares)
        .map_err(|e| ShamirError::Combine(format!("{e:?}")))?;
    <[u8; 32]>::try_from(bytes.as_slice())
        .map_err(|_| ShamirError::Combine("unexpected reconstructed key length".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::shamir_threshold;

    #[test]
    fn empty_committee_has_no_threshold() {
        assert_eq!(shamir_threshold(0), None);
    }

    #[test]
    fn threshold_follows_the_ordinary_quorum() {
        // ARCHITECTURE.md §3.1/§3.4: 1 decides alone, 2 need consensus, N needs N/2 + 1.
        assert_eq!(shamir_threshold(1), Some(1));
        assert_eq!(shamir_threshold(2), Some(2));
        assert_eq!(shamir_threshold(3), Some(2));
        assert_eq!(shamir_threshold(4), Some(3));
    }
}
