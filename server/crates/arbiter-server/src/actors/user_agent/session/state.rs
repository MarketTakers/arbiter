use std::sync::Mutex;

use x25519_dalek::{EphemeralSecret, PublicKey};

pub struct UnsealContext {
    pub client_public_key: PublicKey,
    pub secret: Mutex<Option<EphemeralSecret>>,
}

smlang::statemachine!(
    name: UserAgent,
    custom_error: false,
    transitions: {
        *Idle + UnsealRequest(UnsealContext) / generate_temp_keypair = WaitingForUnsealKey(UnsealContext),
        WaitingForUnsealKey(UnsealContext) + ReceivedValidKey = Unsealed,
        WaitingForUnsealKey(UnsealContext) + ReceivedInvalidKey = Idle,
    }
);

pub struct DummyContext;
impl UserAgentStateMachineContext for DummyContext {
    fn generate_temp_keypair(&mut self, event_data: UnsealContext) -> Result<UnsealContext, ()> {
        Ok(event_data)
    }
}
