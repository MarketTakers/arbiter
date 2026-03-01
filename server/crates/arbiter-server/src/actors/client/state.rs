use arbiter_proto::proto::client::AuthChallenge;
use ed25519_dalek::VerifyingKey;

/// Context for state machine with validated key and sent challenge
#[derive(Clone, Debug)]
pub struct ChallengeContext {
    pub challenge: AuthChallenge,
    pub key: VerifyingKey,
}

smlang::statemachine!(
    name: Client,
    custom_error: false,
    transitions: {
        *Init + AuthRequest = ReceivedAuthRequest,

        ReceivedAuthRequest + SentChallenge(ChallengeContext) / move_challenge = WaitingForChallengeSolution(ChallengeContext),

        WaitingForChallengeSolution(ChallengeContext) + ReceivedGoodSolution = Idle,
        WaitingForChallengeSolution(ChallengeContext) + ReceivedBadSolution = AuthError,
    }
);

pub struct DummyContext;
impl ClientStateMachineContext for DummyContext {
    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    fn move_challenge(&mut self, event_data: ChallengeContext) -> Result<ChallengeContext, ()> {
        Ok(event_data)
    }
}
