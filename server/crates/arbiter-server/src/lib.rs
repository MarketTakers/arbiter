use crate::context::ServerContext;

#[macro_use]
extern crate macro_rules_attribute;

pub mod actors;
pub mod context;
pub mod crypto;
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
