use arbiter_proto::{
    format_challenge,
    proto::user_agent::{
        AuthChallenge, AuthOk,
        UserAgentRequest, UserAgentResponse,
        user_agent_request::Payload as UserAgentRequestPayload,
        user_agent_response::Payload as UserAgentResponsePayload,
    },
    transport::Bi,
};
use arbiter_useragent::UserAgentActor;
use ed25519_dalek::SigningKey;
use kameo::actor::Spawn;
use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};

struct TestTransport {
    inbound_rx: mpsc::Receiver<UserAgentResponse>,
    outbound_tx: mpsc::Sender<UserAgentRequest>,
}

impl Bi<UserAgentResponse, UserAgentRequest> for TestTransport {
    async fn send(&mut self, item: UserAgentRequest) -> Result<(), arbiter_proto::transport::Error> {
        self.outbound_tx
            .send(item)
            .await
            .map_err(|_| arbiter_proto::transport::Error::ChannelClosed)
    }

    async fn recv(&mut self) -> Option<UserAgentResponse> {
        self.inbound_rx.recv().await
    }
}

fn make_transport() -> (
    TestTransport,
    mpsc::Sender<UserAgentResponse>,
    mpsc::Receiver<UserAgentRequest>,
) {
    let (inbound_tx, inbound_rx) = mpsc::channel(8);
    let (outbound_tx, outbound_rx) = mpsc::channel(8);
    (
        TestTransport {
            inbound_rx,
            outbound_tx,
        },
        inbound_tx,
        outbound_rx,
    )
}

fn test_key() -> SigningKey {
    SigningKey::from_bytes(&[7u8; 32])
}

#[tokio::test]
async fn sends_auth_request_on_start_with_bootstrap_token() {
    let key = test_key();
    let pubkey = key.verifying_key().to_bytes().to_vec();
    let bootstrap_token = Some("bootstrap-123".to_string());
    let (transport, inbound_tx, mut outbound_rx) = make_transport();

    let actor = UserAgentActor::spawn(UserAgentActor::new(key, bootstrap_token.clone(), transport));

    let outbound = timeout(Duration::from_secs(1), outbound_rx.recv())
        .await
        .expect("timed out waiting for auth request")
        .expect("channel closed before auth request");

    let UserAgentRequest {
        payload: Some(UserAgentRequestPayload::AuthChallengeRequest(req)),
    } = outbound
    else {
        panic!("expected auth challenge request");
    };

    assert_eq!(req.pubkey, pubkey);
    assert_eq!(req.bootstrap_token, bootstrap_token);

    drop(inbound_tx);
    drop(actor);
}

#[tokio::test]
async fn challenge_flow_sends_solution_from_transport_inbound() {
    let key = test_key();
    let verify_key = key.verifying_key();
    let (transport, inbound_tx, mut outbound_rx) = make_transport();

    let actor = UserAgentActor::spawn(UserAgentActor::new(key, None, transport));

    let _initial_auth_request = timeout(Duration::from_secs(1), outbound_rx.recv())
        .await
        .expect("timed out waiting for initial auth request")
        .expect("missing initial auth request");

    let challenge = AuthChallenge {
        pubkey: verify_key.to_bytes().to_vec(),
        nonce: 42,
    };
    inbound_tx
        .send(UserAgentResponse {
            payload: Some(UserAgentResponsePayload::AuthChallenge(challenge.clone())),
        })
        .await
        .unwrap();

    let outbound = timeout(Duration::from_secs(1), outbound_rx.recv())
        .await
        .expect("timed out waiting for challenge solution")
        .expect("missing challenge solution");

    let UserAgentRequest {
        payload: Some(UserAgentRequestPayload::AuthChallengeSolution(solution)),
    } = outbound
    else {
        panic!("expected auth challenge solution");
    };

    let formatted = format_challenge(challenge.nonce, &challenge.pubkey);
    let sig: ed25519_dalek::Signature = solution
        .signature
        .as_slice()
        .try_into()
        .expect("signature bytes length");
    verify_key
        .verify_strict(&formatted, &sig)
        .expect("solution signature should verify");

    inbound_tx
        .send(UserAgentResponse {
            payload: Some(UserAgentResponsePayload::AuthOk(AuthOk {})),
        })
        .await
        .unwrap();

    drop(inbound_tx);
    drop(actor);
}
