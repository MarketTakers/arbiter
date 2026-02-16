use arbiter_proto::proto::{
    UserAgentResponse,
    auth::{self, AuthChallengeRequest, AuthOk},
    user_agent_response::Payload as UserAgentResponsePayload,
};
use diesel::{ExpressionMethods as _, QueryDsl, insert_into};
use diesel_async::RunQueryDsl;
use ed25519_dalek::Signer as _;
use kameo::actor::Spawn;

use crate::{
    actors::{
        bootstrap::Bootstrapper,
        keyholder::{self, KeyHolder},
        user_agent::{HandleAuthChallengeRequest, HandleAuthChallengeSolution},
    },
    db::{self, schema},
};

use super::UserAgentActor;

#[tokio::test]
#[test_log::test]
pub async fn test_bootstrap_token_auth() {
    let db = db::create_test_pool().await;
    // explicitly not installing any user_agent pubkeys
    let bootstrapper = Bootstrapper::new(&db).await.unwrap(); // this will create bootstrap token
    let keyholder = KeyHolder::new(db.clone()).await.unwrap();
    let token = bootstrapper.get_token().unwrap();

    let bootstrapper_ref = Bootstrapper::spawn(bootstrapper);
    let keyholder_ref = KeyHolder::spawn(keyholder);
    let user_agent = UserAgentActor::new_manual(
        db.clone(),
        bootstrapper_ref,
        keyholder_ref,
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
    // explicitly not installing any user_agent pubkeys
    let bootstrapper = Bootstrapper::new(&db).await.unwrap(); // this will create bootstrap token
    let keyholder = KeyHolder::new(db.clone()).await.unwrap();

    let bootstrapper_ref = Bootstrapper::spawn(bootstrapper);
    let keyholder_ref = KeyHolder::spawn(keyholder);

    let user_agent = UserAgentActor::new_manual(
        db.clone(),
        bootstrapper_ref,
        keyholder_ref,
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

    let bootstrapper_ref = Bootstrapper::spawn(Bootstrapper::new(&db).await.unwrap());
    let keyholder_ref = KeyHolder::spawn(KeyHolder::new(db.clone()).await.unwrap());
    let user_agent = UserAgentActor::new_manual(
        db.clone(),
        bootstrapper_ref,
        keyholder_ref,
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
