use arbiter_proto::proto::user_agent::{
    AuthChallengeRequest, AuthChallengeSolution, KeyType as ProtoKeyType, UserAgentRequest,
    user_agent_request::Payload as UserAgentRequestPayload,
    user_agent_response::Payload as UserAgentResponsePayload,
};
use arbiter_proto::transport::Bi;
use arbiter_server::{
    actors::{
        GlobalActors,
        bootstrap::GetToken,
        user_agent::{UserAgentConnection, connect_user_agent},
    },
    db::{self, schema},
};
use diesel::{ExpressionMethods as _, QueryDsl, insert_into};
use diesel_async::RunQueryDsl;
use ed25519_dalek::Signer as _;

use super::common::ChannelTransport;

#[tokio::test]
#[test_log::test]
pub async fn test_bootstrap_token_auth() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    let token = actors.bootstrapper.ask(GetToken).await.unwrap().unwrap();

    let (server_transport, mut test_transport) = ChannelTransport::new();
    let props = UserAgentConnection::new(db.clone(), actors, Box::new(server_transport));
    let task = tokio::spawn(connect_user_agent(props));

    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    test_transport
        .send(UserAgentRequest {
            payload: Some(UserAgentRequestPayload::AuthChallengeRequest(
                AuthChallengeRequest {
                    pubkey: pubkey_bytes,
                    bootstrap_token: Some(token),
                    key_type: ProtoKeyType::Ed25519.into(),
                },
            )),
        })
        .await
        .unwrap();

    task.await.unwrap();

    let mut conn = db.get().await.unwrap();
    let stored_pubkey: Vec<u8> = schema::useragent_client::table
        .select(schema::useragent_client::public_key)
        .first::<Vec<u8>>(&mut conn)
        .await
        .unwrap();
    assert_eq!(stored_pubkey, new_key.verifying_key().to_bytes().to_vec());
}

#[tokio::test]
#[test_log::test]
pub async fn test_bootstrap_invalid_token_auth() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();

    let (server_transport, mut test_transport) = ChannelTransport::new();
    let props = UserAgentConnection::new(db.clone(), actors, Box::new(server_transport));
    let task = tokio::spawn(connect_user_agent(props));

    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    test_transport
        .send(UserAgentRequest {
            payload: Some(UserAgentRequestPayload::AuthChallengeRequest(
                AuthChallengeRequest {
                    pubkey: pubkey_bytes,
                    bootstrap_token: Some("invalid_token".to_string()),
                    key_type: ProtoKeyType::Ed25519.into(),
                },
            )),
        })
        .await
        .unwrap();

    // Auth fails, connect_user_agent returns, transport drops
    task.await.unwrap();

    // Verify no key was registered
    let mut conn = db.get().await.unwrap();
    let count: i64 = schema::useragent_client::table
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
#[test_log::test]
pub async fn test_challenge_auth() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();

    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    // Pre-register key with key_type
    {
        let mut conn = db.get().await.unwrap();
        insert_into(schema::useragent_client::table)
            .values((
                schema::useragent_client::public_key.eq(pubkey_bytes.clone()),
                schema::useragent_client::key_type.eq(1i32),
            ))
            .execute(&mut conn)
            .await
            .unwrap();
    }

    let (server_transport, mut test_transport) = ChannelTransport::new();
    let props = UserAgentConnection::new(db.clone(), actors, Box::new(server_transport));
    let task = tokio::spawn(connect_user_agent(props));

    // Send challenge request
    test_transport
        .send(UserAgentRequest {
            payload: Some(UserAgentRequestPayload::AuthChallengeRequest(
                AuthChallengeRequest {
                    pubkey: pubkey_bytes,
                    bootstrap_token: None,
                    key_type: ProtoKeyType::Ed25519.into(),
                },
            )),
        })
        .await
        .unwrap();

    // Read the challenge response
    let response = test_transport
        .recv()
        .await
        .expect("should receive challenge");
    let challenge = match response {
        Ok(resp) => match resp.payload {
            Some(UserAgentResponsePayload::AuthChallenge(c)) => c,
            other => panic!("Expected AuthChallenge, got {other:?}"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    };

    // Sign the challenge and send solution
    let formatted_challenge = arbiter_proto::format_challenge(challenge.nonce, &challenge.pubkey);
    let signature = new_key.sign(&formatted_challenge);

    test_transport
        .send(UserAgentRequest {
            payload: Some(UserAgentRequestPayload::AuthChallengeSolution(
                AuthChallengeSolution {
                    signature: signature.to_bytes().to_vec(),
                },
            )),
        })
        .await
        .unwrap();

    // Auth completes, session spawned
    task.await.unwrap();
}
