use alloy::primitives::{Address, ChainId, address};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenInfo {
    pub name: &'static str,
    pub symbol: &'static str,
    pub decimals: u32,
    pub contract: Address,
    pub chain: ChainId,
    pub logo_uri: Option<&'static str>,
}

include!("tokens.rs");
