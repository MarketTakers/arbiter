use arbiter_proto::proto::{
    UnsealEncryptedKey, UnsealResult, UnsealStart, UserAgentResponse,
    auth::{self, AuthChallengeRequest, AuthOk},
    user_agent_response::Payload as UserAgentResponsePayload,
};
use chacha20poly1305::{AeadInPlace, XChaCha20Poly1305, XNonce, aead::KeyInit};
use diesel::{ExpressionMethods as _, QueryDsl, insert_into};
use diesel_async::RunQueryDsl;
use ed25519_dalek::Signer as _;
use kameo::actor::{ActorRef, Spawn};
use memsafe::MemSafe;
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::{
    actors::{
        GlobalActors,
        bootstrap::GetToken,
        keyholder::{Bootstrap, Seal},
        user_agent::{
            HandleAuthChallengeRequest, HandleAuthChallengeSolution, HandleUnsealEncryptedKey,
            HandleUnsealRequest,
        },
    },
    db::{self, models::ArbiterSetting, schema},
};

use super::UserAgentActor;

async fn seed_settings(db: &db::DatabasePool) {
    let mut conn = db.get().await.unwrap();
    insert_into(schema::arbiter_settings::table)
        .values(&ArbiterSetting {
            id: 1,
            root_key_id: None,
            cert_key: vec![],
            cert: vec![],
        })
        .execute(&mut conn)
        .await
        .unwrap();
}

/// Bootstrap keyholder with `seal_key`, and Seal it
/// then create and authenticate a user agent (reaching Idle state).
async fn setup_authenticated_user_agent(
    seal_key: &[u8],
) -> (db::DatabasePool, ActorRef<UserAgentActor>) {
    let db = db::create_test_pool().await;
    seed_settings(&db).await;

    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .key_holder
        .ask(Bootstrap {
            seal_key_raw: MemSafe::new(seal_key.to_vec()).unwrap(),
        })
        .await
        .unwrap();
    actors.key_holder.ask(Seal).await.unwrap();

    let user_agent = UserAgentActor::new_manual(
        db.clone(),
        actors.clone(),
        tokio::sync::mpsc::channel(1).0,
    );
    let user_agent_ref = UserAgentActor::spawn(user_agent);
    let token = actors.bootstrapper.ask(GetToken).await.unwrap().unwrap();

    let auth_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    user_agent_ref
        .ask(HandleAuthChallengeRequest {
            req: AuthChallengeRequest {
                pubkey: auth_key.verifying_key().to_bytes().to_vec(),
                bootstrap_token: Some(token),
            },
        })
        .await
        .unwrap();

    (db, user_agent_ref)
}

/// Client side of the DH unseal exchange:
/// sends UnsealStart, derives shared secret, encrypts `key_to_send`.
async fn client_dh_encrypt(
    user_agent_ref: &ActorRef<UserAgentActor>,
    key_to_send: &[u8],
) -> UnsealEncryptedKey {
    let client_secret = EphemeralSecret::random();
    let client_public = PublicKey::from(&client_secret);

    let response = user_agent_ref
        .ask(HandleUnsealRequest {
            req: UnsealStart {
                client_pubkey: client_public.as_bytes().to_vec(),
            },
        })
        .await
        .unwrap();

    let server_pubkey = match response.payload.unwrap() {
        UserAgentResponsePayload::UnsealStartResponse(resp) => resp.server_pubkey,
        other => panic!("Expected UnsealStartResponse, got {other:?}"),
    };
    let server_public = PublicKey::from(
        <[u8; 32]>::try_from(server_pubkey.as_slice()).unwrap(),
    );

    let shared_secret = client_secret.diffie_hellman(&server_public);
    let cipher = XChaCha20Poly1305::new(shared_secret.as_bytes().into());
    let nonce = XNonce::from([0u8; 24]);
    let associated_data = b"unseal";
    let mut ciphertext = key_to_send.to_vec();
    cipher
        .encrypt_in_place(&nonce, associated_data, &mut ciphertext)
        .unwrap();

    UnsealEncryptedKey {
        nonce: nonce.to_vec(),
        ciphertext,
        associated_data: associated_data.to_vec(),
    }
}

#[tokio::test]
#[test_log::test]
pub async fn test_bootstrap_token_auth() {
    let db = db::create_test_pool().await;
    seed_settings(&db).await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    let token = actors.bootstrapper.ask(GetToken).await.unwrap().unwrap();
    let user_agent = UserAgentActor::new_manual(
        db.clone(),
        actors.clone(),
        tokio::sync::mpsc::channel(1).0, // dummy channel, we won't actually send responses in this test
    );
    let user_agent_ref = UserAgentActor::spawn(user_agent);

    // simulate client sending auth request with bootstrap token
    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    let result = user_agent_ref
        .ask(HandleAuthChallengeRequest {
            req: AuthChallengeRequest {
                pubkey: pubkey_bytes,
                bootstrap_token: Some(token),
            },
        })
        .await
        .expect("Shouldn't fail to send message");

    // auth succeeded
    assert_eq!(
        result,
        UserAgentResponse {
            payload: Some(UserAgentResponsePayload::AuthMessage(
                arbiter_proto::proto::auth::ServerMessage {
                    payload: Some(arbiter_proto::proto::auth::server_message::Payload::AuthOk(
                        AuthOk {},
                    )),
                },
            )),
        }
    );

    // key is succesfully recorded in database
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
    seed_settings(&db).await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();

    let user_agent = UserAgentActor::new_manual(
        db.clone(),
        actors,
        tokio::sync::mpsc::channel(1).0, // dummy channel, we won't actually send responses in this test
    );
    let user_agent_ref = UserAgentActor::spawn(user_agent);

    // simulate client sending auth request with bootstrap token
    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    let result = user_agent_ref
        .ask(HandleAuthChallengeRequest {
            req: AuthChallengeRequest {
                pubkey: pubkey_bytes,
                bootstrap_token: Some("invalid_token".to_string()),
            },
        })
        .await;

    match result {
        Err(kameo::error::SendError::HandlerError(status)) => {
            assert_eq!(status.code(), tonic::Code::InvalidArgument);
            insta::assert_debug_snapshot!(status, @r#"
                Status {
                    code: InvalidArgument,
                    message: "Invalid bootstrap token",
                    source: None,
                }
                "#);
        }
        Err(other) => {
            panic!("Expected SendError::HandlerError, got {other:?}");
        }
        Ok(_) => {
            panic!("Expected error due to invalid bootstrap token, but got success");
        }
    }
}

#[tokio::test]
#[test_log::test]
pub async fn test_challenge_auth() {
    let db = db::create_test_pool().await;
    seed_settings(&db).await;

    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    let user_agent = UserAgentActor::new_manual(
        db.clone(),
        actors,
        tokio::sync::mpsc::channel(1).0, // dummy channel, we won't actually send responses in this test
    );
    let user_agent_ref = UserAgentActor::spawn(user_agent);

    // simulate client sending auth request with bootstrap token
    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    // insert pubkey into database to trigger challenge-response auth flow
    {
        let mut conn = db.get().await.unwrap();
        insert_into(schema::useragent_client::table)
            .values(schema::useragent_client::public_key.eq(pubkey_bytes.clone()))
            .execute(&mut conn)
            .await
            .unwrap();
    }

    let result = user_agent_ref
        .ask(HandleAuthChallengeRequest {
            req: AuthChallengeRequest {
                pubkey: pubkey_bytes,
                bootstrap_token: None,
            },
        })
        .await
        .expect("Shouldn't fail to send message");

    // auth challenge succeeded
    let UserAgentResponse {
        payload:
            Some(UserAgentResponsePayload::AuthMessage(arbiter_proto::proto::auth::ServerMessage {
                payload:
                    Some(arbiter_proto::proto::auth::server_message::Payload::AuthChallenge(challenge)),
            })),
    } = result
    else {
        panic!("Expected auth challenge response, got {result:?}");
    };

    let formatted_challenge = arbiter_proto::format_challenge(&challenge);
    let signature = new_key.sign(&formatted_challenge);
    let serialized_signature = signature.to_bytes().to_vec();

    let result = user_agent_ref
        .ask(HandleAuthChallengeSolution {
            solution: auth::AuthChallengeSolution {
                signature: serialized_signature,
            },
        })
        .await
        .expect("Shouldn't fail to send message");

    // auth succeeded
    assert_eq!(
        result,
        UserAgentResponse {
            payload: Some(UserAgentResponsePayload::AuthMessage(
                arbiter_proto::proto::auth::ServerMessage {
                    payload: Some(arbiter_proto::proto::auth::server_message::Payload::AuthOk(
                        AuthOk {},
                    )),
                },
            )),
        }
    );
}

#[tokio::test]
#[test_log::test]
pub async fn test_unseal_success() {
    let seal_key = b"test-seal-key";
    let (_db, user_agent_ref) = setup_authenticated_user_agent(seal_key).await;

    let encrypted_key = client_dh_encrypt(&user_agent_ref, seal_key).await;

    let response = user_agent_ref
        .ask(HandleUnsealEncryptedKey { req: encrypted_key })
        .await
        .unwrap();

    assert_eq!(
        response.payload.unwrap(),
        UserAgentResponsePayload::UnsealResult(UnsealResult::Success.into()),
    );
}

#[tokio::test]
#[test_log::test]
pub async fn test_unseal_wrong_seal_key() {
    let (_db, user_agent_ref) = setup_authenticated_user_agent(b"correct-key").await;

    // Encrypt a different key through the DH channel
    let encrypted_key = client_dh_encrypt(&user_agent_ref, b"wrong-key").await;

    let response = user_agent_ref
        .ask(HandleUnsealEncryptedKey { req: encrypted_key })
        .await
        .unwrap();

    assert_eq!(
        response.payload.unwrap(),
        UserAgentResponsePayload::UnsealResult(UnsealResult::InvalidKey.into()),
    );
}

#[tokio::test]
#[test_log::test]
pub async fn test_unseal_corrupted_ciphertext() {
    let (_db, user_agent_ref) = setup_authenticated_user_agent(b"test-key").await;

    // Do UnsealStart to reach WaitingForUnsealKey state
    let client_secret = EphemeralSecret::random();
    let client_public = PublicKey::from(&client_secret);

    user_agent_ref
        .ask(HandleUnsealRequest {
            req: UnsealStart {
                client_pubkey: client_public.as_bytes().to_vec(),
            },
        })
        .await
        .unwrap();

    // Send garbage that wasn't encrypted with the DH shared secret
    let response = user_agent_ref
        .ask(HandleUnsealEncryptedKey {
            req: UnsealEncryptedKey {
                nonce: vec![0u8; 24],
                ciphertext: vec![0u8; 32],
                associated_data: vec![],
            },
        })
        .await
        .unwrap();

    assert_eq!(
        response.payload.unwrap(),
        UserAgentResponsePayload::UnsealResult(UnsealResult::InvalidKey.into()),
    );
}

#[tokio::test]
#[test_log::test]
pub async fn test_unseal_start_without_auth_fails() {
    let db = db::create_test_pool().await;
    seed_settings(&db).await;

    let actors = GlobalActors::spawn(db.clone()).await.unwrap();

    let user_agent = UserAgentActor::new_manual(
        db.clone(),
        actors,
        tokio::sync::mpsc::channel(1).0,
    );
    let user_agent_ref = UserAgentActor::spawn(user_agent);

    // Try unseal from Init state (not authenticated)
    let client_secret = EphemeralSecret::random();
    let client_public = PublicKey::from(&client_secret);

    let result = user_agent_ref
        .ask(HandleUnsealRequest {
            req: UnsealStart {
                client_pubkey: client_public.as_bytes().to_vec(),
            },
        })
        .await;

    match result {
        Err(kameo::error::SendError::HandlerError(status)) => {
            assert_eq!(status.code(), tonic::Code::Internal);
        }
        other => panic!("Expected state machine error, got {other:?}"),
    }
}

#[tokio::test]
#[test_log::test]
pub async fn test_unseal_retry_after_invalid_key() {
    let seal_key = b"real-seal-key";
    let (_db, user_agent_ref) = setup_authenticated_user_agent(seal_key).await;

    // First attempt: wrong key -> InvalidKey, state goes back to Idle
    {
        let encrypted_key = client_dh_encrypt(&user_agent_ref, b"wrong-key").await;

        let response = user_agent_ref
            .ask(HandleUnsealEncryptedKey { req: encrypted_key })
            .await
            .unwrap();

        assert_eq!(
            response.payload.unwrap(),
            UserAgentResponsePayload::UnsealResult(UnsealResult::InvalidKey.into()),
        );
    }

    // Second attempt: correct key -> Success
    {
        let encrypted_key = client_dh_encrypt(&user_agent_ref, seal_key).await;

        let response = user_agent_ref
            .ask(HandleUnsealEncryptedKey { req: encrypted_key })
            .await
            .unwrap();

        assert_eq!(
            response.payload.unwrap(),
            UserAgentResponsePayload::UnsealResult(UnsealResult::Success.into()),
        );
    }
}
