use arbiter_proto::{
    proto::{ClientRequest, ClientResponse},
    transport::Bi,
};

use crate::ServerContext;

pub(crate) async fn handle_client(
    _context: ServerContext,
    _bistream: impl Bi<ClientRequest, ClientResponse>,
) {
}
