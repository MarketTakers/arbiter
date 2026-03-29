use alloy::{consensus::TxEip1559, primitives::Address, rlp::Decodable as _};

use crate::grpc::TryConvert;

pub struct RawEvmAddress(pub Vec<u8>);
impl TryConvert for RawEvmAddress {
    type Output = Address;

    type Error = tonic::Status;

    fn try_convert(self) -> Result<Self::Output, Self::Error> {
        let wallet_address = match <[u8; 20]>::try_from(self.0.as_slice()) {
            Ok(address) => Address::from(address),
            Err(_) => {
                return Err(tonic::Status::invalid_argument(
                    "Invalid EVM wallet address",
                ));
            }
        };
        Ok(wallet_address)
    }
}

pub struct RawEvmTransaction(pub Vec<u8>);
impl TryConvert for RawEvmTransaction {
    type Output = TxEip1559;

    type Error = tonic::Status;

    fn try_convert(mut self) -> Result<Self::Output, Self::Error> {
        let tx = TxEip1559::decode(&mut self.0.as_slice()).map_err(|_| {
            tonic::Status::invalid_argument("Invalid EVM transaction format")
        })?;
        Ok(tx)
    }
}