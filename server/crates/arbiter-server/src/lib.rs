#![forbid(unsafe_code)]
use crate::context::ServerContext;

pub mod actors;
pub mod context;
pub mod db;
pub mod evm;
pub mod grpc;
pub mod safe_cell;
pub mod utils;

pub struct Server {
    context: ServerContext,
}

impl Server {
    pub fn new(context: ServerContext) -> Self {
        Self { context }
    }
}
