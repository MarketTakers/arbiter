use x25519_dalek::{PublicKey, SharedSecret};

#[derive(Default)]
pub enum State {
    #[default]
    Idle,
    ReadyForExchange {
        server_key: PublicKey,
        secret: SharedSecret,
    },
}
