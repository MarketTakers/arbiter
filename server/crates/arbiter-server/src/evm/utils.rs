use alloy::primitives::U256;

pub fn u256_to_bytes(value: U256) -> [u8; 32] {
    value.to_le_bytes()
}
pub fn bytes_to_u256(bytes: &[u8]) -> Option<U256> {
    let bytes: [u8; 32] = bytes.try_into().ok()?;
    Some(U256::from_le_bytes(bytes))
}
