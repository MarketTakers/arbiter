use crate::context::ServerContext;

pub mod actors;
pub mod context;
pub mod crypto;
pub mod db;
pub mod evm;
pub mod grpc;
pub mod peers;
pub mod utils;

pub struct Server {
    context: ServerContext,
}

impl Server {
    pub const fn new(context: ServerContext) -> Self {
        Self { context }
    }
}
