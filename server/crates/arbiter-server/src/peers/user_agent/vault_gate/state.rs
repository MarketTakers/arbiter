use std::sync::Mutex;

use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};



pub struct Handshake {
    client_pubkey: PublicKey,
}



#[derive(Default)]
pub enum State {
    #[default]
    Idle, 
    ReadyForExchange { server_key: PublicKey, secret: SharedSecret },
}