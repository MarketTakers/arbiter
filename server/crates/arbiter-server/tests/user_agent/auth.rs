use arbiter_proto::transport::{Receiver, Sender};
use arbiter_server::{
    actors::{
        GlobalActors,
        bootstrap::GetToken,
        keyholder::Bootstrap,
        user_agent::{AuthPublicKey, UserAgentConnection, auth},
    },
    db::{self, schema},
    safe_cell::{SafeCell, SafeCellHandle as _},
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
    let db_for_task = db.clone();
    let task = tokio::spawn(async move {
        let mut props = UserAgentConnection::new(db_for_task, actors);
        auth::authenticate(&mut props, server_transport).await
    });

    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    test_transport
        .send(auth::Inbound::AuthChallengeRequest {
            pubkey: AuthPublicKey::Ed25519(new_key.verifying_key()),
            bootstrap_token: Some(token),
        })
        .await
        .unwrap();

    let response = test_transport
        .recv()
        .await
        .expect("should receive auth result");
    match response {
        Ok(auth::Outbound::AuthSuccess) => {}
        other => panic!("Expected AuthSuccess, got {other:?}"),
    }

    task.await.unwrap().unwrap();

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
    let db_for_task = db.clone();
    let task = tokio::spawn(async move {
        let mut props = UserAgentConnection::new(db_for_task, actors);
        auth::authenticate(&mut props, server_transport).await
    });

    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    test_transport
        .send(auth::Inbound::AuthChallengeRequest {
            pubkey: AuthPublicKey::Ed25519(new_key.verifying_key()),
            bootstrap_token: Some("invalid_token".to_string()),
        })
        .await
        .unwrap();

    assert!(matches!(
        task.await.unwrap(),
        Err(auth::Error::InvalidBootstrapToken)
    ));

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
    let db_for_task = db.clone();
    let task = tokio::spawn(async move {
        let mut props = UserAgentConnection::new(db_for_task, actors);
        auth::authenticate(&mut props, server_transport).await
    });

    // Send challenge request
    test_transport
        .send(auth::Inbound::AuthChallengeRequest {
            pubkey: AuthPublicKey::Ed25519(new_key.verifying_key()),
            bootstrap_token: None,
        })
        .await
        .unwrap();

    // Read the challenge response
    let response = test_transport
        .recv()
        .await
        .expect("should receive challenge");
    let challenge = match response {
        Ok(resp) => match resp {
            auth::Outbound::AuthChallenge { nonce } => nonce,
            other => panic!("Expected AuthChallenge, got {other:?}"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    };

    let formatted_challenge = arbiter_proto::format_challenge(challenge, &pubkey_bytes);
    let signature = new_key.sign(&formatted_challenge);

    test_transport
        .send(auth::Inbound::AuthChallengeSolution {
            signature: signature.to_bytes().to_vec(),
        })
        .await
        .unwrap();

    let response = test_transport
        .recv()
        .await
        .expect("should receive auth result");
    match response {
        Ok(auth::Outbound::AuthSuccess) => {}
        other => panic!("Expected AuthSuccess, got {other:?}"),
    }

    task.await.unwrap().unwrap();
}

#[tokio::test]
#[test_log::test]
pub async fn test_challenge_auth_rejects_integrity_tag_mismatch_when_unsealed() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();

    actors
        .key_holder
        .ask(Bootstrap {
            seal_key_raw: SafeCell::new(b"test-seal-key".to_vec()),
        })
        .await
        .unwrap();

    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    {
        let mut conn = db.get().await.unwrap();
        insert_into(schema::useragent_client::table)
            .values((
                schema::useragent_client::public_key.eq(pubkey_bytes.clone()),
                schema::useragent_client::key_type.eq(1i32),
                schema::useragent_client::pubkey_integrity_tag.eq(Some(vec![0u8; 32])),
            ))
            .execute(&mut conn)
            .await
            .unwrap();
    }

    let (server_transport, mut test_transport) = ChannelTransport::new();
    let db_for_task = db.clone();
    let task = tokio::spawn(async move {
        let mut props = UserAgentConnection::new(db_for_task, actors);
        auth::authenticate(&mut props, server_transport).await
    });

    test_transport
        .send(auth::Inbound::AuthChallengeRequest {
            pubkey: AuthPublicKey::Ed25519(new_key.verifying_key()),
            bootstrap_token: None,
        })
        .await
        .unwrap();

    assert!(matches!(
        task.await.unwrap(),
        Err(auth::Error::InvalidChallengeSolution)
    ));
}

#[tokio::test]
#[test_log::test]
pub async fn test_challenge_auth_rejects_invalid_signature() {
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
    let db_for_task = db.clone();
    let task = tokio::spawn(async move {
        let mut props = UserAgentConnection::new(db_for_task, actors);
        auth::authenticate(&mut props, server_transport).await
    });

    test_transport
        .send(auth::Inbound::AuthChallengeRequest {
            pubkey: AuthPublicKey::Ed25519(new_key.verifying_key()),
            bootstrap_token: None,
        })
        .await
        .unwrap();

    let response = test_transport
        .recv()
        .await
        .expect("should receive challenge");
    let challenge = match response {
        Ok(resp) => match resp {
            auth::Outbound::AuthChallenge { nonce } => nonce,
            other => panic!("Expected AuthChallenge, got {other:?}"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    };

    // Sign a different challenge value so signature format is valid but verification must fail.
    let wrong_challenge = arbiter_proto::format_challenge(challenge + 1, &pubkey_bytes);
    let signature = new_key.sign(&wrong_challenge);

    test_transport
        .send(auth::Inbound::AuthChallengeSolution {
            signature: signature.to_bytes().to_vec(),
        })
        .await
        .unwrap();

    let expected_err = task.await.unwrap();

    println!("Received expected error: {expected_err:#?}");

    assert!(matches!(
        expected_err,
        Err(auth::Error::InvalidChallengeSolution)
    ));
}
