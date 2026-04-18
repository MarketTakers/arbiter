use alloy::primitives::U256;

#[derive(thiserror::Error, Debug)]
#[error("Expected {expected} bytes but got {actual} bytes")]
pub(super) struct LengthError {
    pub(super) expected: usize,
    pub(super) actual: usize,
}

pub const fn u256_to_bytes(value: U256) -> [u8; 32] {
    value.to_le_bytes()
}
pub(super) fn bytes_to_u256(bytes: &[u8]) -> Option<U256> {
    let bytes: [u8; 32] = bytes.try_into().ok()?;
    Some(U256::from_le_bytes(bytes))
}

pub(super) fn try_bytes_to_u256(bytes: &[u8]) -> diesel::result::QueryResult<U256> {
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| {
        diesel::result::Error::DeserializationError(Box::new(LengthError {
            expected: 32,
            actual: bytes.len(),
        }))
    })?;
    Ok(U256::from_le_bytes(bytes))
}
