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



pub struct UnsealContext {
    pub client_public_key: PublicKey,
    pub secret: Mutex<Option<EphemeralSecret>>,
}



smlang::statemachine!(
    name: UserAgent,
    custom_error: false,
    transitions: {
        *Init + AuthRequest =  ReceivedAuthRequest,
        ReceivedAuthRequest + ReceivedBootstrapToken = Idle,

        ReceivedAuthRequest + SentChallenge(ChallengeContext) / move_challenge = WaitingForChallengeSolution(ChallengeContext),

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
    fn generate_temp_keypair(&mut self, event_data: UnsealContext) -> Result<UnsealContext, ()> {
        Ok(event_data)
    }
    
    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    fn move_challenge< >(&mut self,event_data:ChallengeContext) -> Result<ChallengeContext,()>  {
        Ok(event_data)
    }
}
