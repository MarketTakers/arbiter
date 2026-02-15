use std::sync::Mutex;

use arbiter_proto::proto::auth::AuthChallenge;
use ed25519_dalek::VerifyingKey;
use x25519_dalek::{EphemeralSecret, PublicKey};

/// Context for state machine with validated key and sent challenge
/// Challenge is then transformed to bytes using shared function and verified
#[derive(Clone, Debug)]
pub struct ChallengeContext {
    pub challenge: AuthChallenge,
    pub key: VerifyingKey,
}

// Request context with deserialized public key for state machine.
// This intermediate struct is needed because the state machine branches depending on presence of bootstrap token,
// but we want to have the deserialized key in both branches.
#[derive(Clone, Debug)]
pub struct AuthRequestContext {
    pub pubkey: VerifyingKey,
    pub bootstrap_token: Option<String>,
}

pub struct UnsealContext {
    pub server_public_key: PublicKey,
    pub client_public_key: PublicKey,
    pub secret: Mutex<Option<EphemeralSecret>>,
}



smlang::statemachine!(
    name: UserAgent,
    custom_error: false,
    transitions: {
        *Init + AuthRequest(AuthRequestContext) / auth_request_context =  ReceivedAuthRequest(AuthRequestContext),
        ReceivedAuthRequest(AuthRequestContext) + ReceivedBootstrapToken = Idle,

        ReceivedAuthRequest(AuthRequestContext) + SentChallenge(ChallengeContext) / move_challenge = WaitingForChallengeSolution(ChallengeContext),

        WaitingForChallengeSolution(ChallengeContext) + ReceivedGoodSolution = Idle,
        WaitingForChallengeSolution(ChallengeContext) + ReceivedBadSolution = AuthError, // block further transitions, but connection should close anyway

        Idle + UnsealRequest(UnsealContext) / generate_temp_keypair = WaitingForUnsealKey(UnsealContext),
        WaitingForUnsealKey(UnsealContext) + ReceivedValidKey = Unsealed,
        WaitingForUnsealKey(UnsealContext) + ReceivedInvalidKey = Idle,
    }
);

pub struct DummyContext;
impl UserAgentStateMachineContext for DummyContext {
    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    fn move_challenge(
        &mut self,
        _state_data: &AuthRequestContext,
        event_data: ChallengeContext,
    ) -> Result<ChallengeContext, ()> {
        Ok(event_data)
    }

    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    fn auth_request_context(
        &mut self,
        event_data: AuthRequestContext,
    ) -> Result<AuthRequestContext, ()> {
        Ok(event_data)
    }

    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    fn generate_temp_keypair(&mut self, event_data: UnsealContext) -> Result<UnsealContext, ()> {
        Ok(event_data)
    }
}
