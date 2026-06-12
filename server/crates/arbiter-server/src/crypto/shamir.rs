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

/// Reconstruct the secret from `threshold` or more shares.
pub fn combine_shares(shares: &[Vec<u8>]) -> Result<[u8; 32], ShamirError> {
    let bytes = Gf256::combine_array(shares)
        .map_err(|e| ShamirError::Combine(format!("{e:?}")))?;
    <[u8; 32]>::try_from(bytes.as_slice())
        .map_err(|_| ShamirError::Combine("unexpected reconstructed key length".to_owned()))
}
