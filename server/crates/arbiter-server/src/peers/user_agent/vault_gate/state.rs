
use x25519_dalek::{PublicKey, SharedSecret};

pub struct Handshake {
    client_pubkey: PublicKey,
}

#[derive(Default)]
pub enum State {
    #[default]
    Idle,
    ReadyForExchange {
        server_key: PublicKey,
        secret: SharedSecret,
    },
}
