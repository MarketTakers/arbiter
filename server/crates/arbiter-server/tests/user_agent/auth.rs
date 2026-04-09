use arbiter_crypto::{
    authn::{self, USERAGENT_CONTEXT, format_challenge},
    safecell::{SafeCell, SafeCellHandle as _},
};

use arbiter_proto::transport::{Receiver, Sender};
use arbiter_server::{
    actors::{
        GlobalActors,
        bootstrap::GetToken,
        keyholder::Bootstrap,
        user_agent::{UserAgentConnection, UserAgentCredentials, auth},
    },
    crypto::integrity,
    db::{self, schema},
};
use diesel::{ExpressionMethods as _, QueryDsl, insert_into};
use diesel_async::RunQueryDsl;
use ml_dsa::{KeyGen, MlDsa87, SigningKey, VerifyingKey, signature::Keypair};

use super::common::ChannelTransport;

fn verifying_key(key: &SigningKey<MlDsa87>) -> VerifyingKey<MlDsa87> {
    <SigningKey<MlDsa87> as Keypair>::verifying_key(key)
}

fn sign_useragent_challenge(
    key: &SigningKey<MlDsa87>,
    nonce: i32,
    pubkey_bytes: &[u8],
) -> authn::Signature {
    let challenge = format_challenge(nonce, pubkey_bytes);
    key.signing_key()
        .sign_deterministic(&challenge, USERAGENT_CONTEXT)
        .unwrap()
        .into()
}

#[tokio::test]
#[test_log::test]
pub async fn bootstrap_token_auth() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .key_holder
        .ask(Bootstrap {
            seal_key_raw: SafeCell::new(b"test-seal-key".to_vec()),
        })
        .await
        .unwrap();
    let token = actors.bootstrapper.ask(GetToken).await.unwrap().unwrap();

    let (server_transport, mut test_transport) = ChannelTransport::new();
    let db_for_task = db.clone();
    let task = tokio::spawn(async move {
        let mut props = UserAgentConnection::new(db_for_task, actors);
        auth::authenticate(&mut props, server_transport).await
    });

    let new_key = MlDsa87::key_gen(&mut rand::rng());
    test_transport
        .send(auth::Inbound::AuthChallengeRequest {
            pubkey: verifying_key(&new_key).into(),
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
    assert_eq!(stored_pubkey, verifying_key(&new_key).encode().0.to_vec());
}

#[tokio::test]
#[test_log::test]
pub async fn bootstrap_invalid_token_auth() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();

    let (server_transport, mut test_transport) = ChannelTransport::new();
    let db_for_task = db.clone();
    let task = tokio::spawn(async move {
        let mut props = UserAgentConnection::new(db_for_task, actors);
        auth::authenticate(&mut props, server_transport).await
    });

    let new_key = MlDsa87::key_gen(&mut rand::rng());
    test_transport
        .send(auth::Inbound::AuthChallengeRequest {
            pubkey: verifying_key(&new_key).into(),
            bootstrap_token: Some("invalid_token".to_owned()),
        })
        .await
        .unwrap();

    assert!(matches!(
        task.await.unwrap(),
        Err(auth::Error::InvalidBootstrapToken)
    ));

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
pub async fn challenge_auth() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .key_holder
        .ask(Bootstrap {
            seal_key_raw: SafeCell::new(b"test-seal-key".to_vec()),
        })
        .await
        .unwrap();

    let new_key = MlDsa87::key_gen(&mut rand::rng());
    let pubkey_bytes = authn::PublicKey::from(verifying_key(&new_key)).to_bytes();

    {
        let mut conn = db.get().await.unwrap();
        let id: i32 = insert_into(schema::useragent_client::table)
            .values((
                schema::useragent_client::public_key.eq(pubkey_bytes.clone()),
                schema::useragent_client::key_type.eq(1i32),
            ))
            .returning(schema::useragent_client::id)
            .get_result(&mut conn)
            .await
            .unwrap();
        integrity::sign_entity(
            &mut conn,
            &actors.key_holder,
            &UserAgentCredentials {
                pubkey: verifying_key(&new_key).into(),
                nonce: 1,
            },
            id,
        )
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
            pubkey: verifying_key(&new_key).into(),
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
            auth::Outbound::AuthSuccess => panic!("Expected AuthChallenge, got AuthSuccess"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    };

    let signature = sign_useragent_challenge(&new_key, challenge, &pubkey_bytes);

    test_transport
        .send(auth::Inbound::AuthChallengeSolution {
            signature: signature.to_bytes(),
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
pub async fn challenge_auth_rejects_integrity_tag_mismatch_when_unsealed() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();

    actors
        .key_holder
        .ask(Bootstrap {
            seal_key_raw: SafeCell::new(b"test-seal-key".to_vec()),
        })
        .await
        .unwrap();

    let new_key = MlDsa87::key_gen(&mut rand::rng());
    let pubkey_bytes = authn::PublicKey::from(verifying_key(&new_key)).to_bytes();

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
            pubkey: verifying_key(&new_key).into(),
            bootstrap_token: None,
        })
        .await
        .unwrap();

    assert!(matches!(
        task.await.unwrap(),
        Err(auth::Error::Internal { .. })
    ));
}

#[tokio::test]
#[test_log::test]
pub async fn challenge_auth_rejects_invalid_signature() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .key_holder
        .ask(Bootstrap {
            seal_key_raw: SafeCell::new(b"test-seal-key".to_vec()),
        })
        .await
        .unwrap();

    let new_key = MlDsa87::key_gen(&mut rand::rng());
    let pubkey_bytes = authn::PublicKey::from(verifying_key(&new_key)).to_bytes();

    {
        let mut conn = db.get().await.unwrap();
        let id: i32 = insert_into(schema::useragent_client::table)
            .values((
                schema::useragent_client::public_key.eq(pubkey_bytes.clone()),
                schema::useragent_client::key_type.eq(1i32),
            ))
            .returning(schema::useragent_client::id)
            .get_result(&mut conn)
            .await
            .unwrap();
        integrity::sign_entity(
            &mut conn,
            &actors.key_holder,
            &UserAgentCredentials {
                pubkey: verifying_key(&new_key).into(),
                nonce: 1,
            },
            id,
        )
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
            pubkey: verifying_key(&new_key).into(),
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
            auth::Outbound::AuthSuccess => panic!("Expected AuthChallenge, got AuthSuccess"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    };

    let signature = sign_useragent_challenge(&new_key, challenge + 1, &pubkey_bytes);

    test_transport
        .send(auth::Inbound::AuthChallengeSolution {
            signature: signature.to_bytes(),
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
