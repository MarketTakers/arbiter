use arbiter_proto::proto::{
    UserAgentResponse,
    UserAgentRequest,
    auth::{self, AuthChallengeRequest, AuthOk, ClientMessage, client_message::Payload as ClientAuthPayload},
    user_agent_request::Payload as UserAgentRequestPayload,
    user_agent_response::Payload as UserAgentResponsePayload,
};
use arbiter_server::{
    actors::{
        GlobalActors,
        bootstrap::GetToken,
        user_agent::{UserAgentActor, UserAgentError},
    },
    db::{self, schema},
};
use diesel::{ExpressionMethods as _, QueryDsl, insert_into};
use diesel_async::RunQueryDsl;
use ed25519_dalek::Signer as _;

fn auth_request(payload: ClientAuthPayload) -> UserAgentRequest {
    UserAgentRequest {
        payload: Some(UserAgentRequestPayload::AuthMessage(ClientMessage {
            payload: Some(payload),
        })),
    }
}

#[tokio::test]
#[test_log::test]
pub async fn test_bootstrap_token_auth() {
    let db =db::create_test_pool().await;

    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    let token = actors.bootstrapper.ask(GetToken).await.unwrap().unwrap();
    let mut user_agent = UserAgentActor::new_manual(db.clone(), actors);

    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    let result = user_agent
        .process_transport_inbound(auth_request(ClientAuthPayload::AuthChallengeRequest(
            AuthChallengeRequest {
                pubkey: pubkey_bytes,
                bootstrap_token: Some(token),
            },
        )))
        .await
        .expect("Shouldn't fail to process message");

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
    let mut user_agent = UserAgentActor::new_manual(db.clone(), actors);

    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    let result = user_agent
        .process_transport_inbound(auth_request(ClientAuthPayload::AuthChallengeRequest(
            AuthChallengeRequest {
                pubkey: pubkey_bytes,
                bootstrap_token: Some("invalid_token".to_string()),
            },
        )))
        .await;

    match result {
        Err(err) => {
            assert_eq!(err, UserAgentError::InvalidBootstrapToken);
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

    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    let mut user_agent = UserAgentActor::new_manual(db.clone(), actors);

    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    {
        let mut conn = db.get().await.unwrap();
        insert_into(schema::useragent_client::table)
            .values(schema::useragent_client::public_key.eq(pubkey_bytes.clone()))
            .execute(&mut conn)
            .await
            .unwrap();
    }

    let result = user_agent
        .process_transport_inbound(auth_request(ClientAuthPayload::AuthChallengeRequest(
            AuthChallengeRequest {
                pubkey: pubkey_bytes,
                bootstrap_token: None,
            },
        )))
        .await
        .expect("Shouldn't fail to process message");

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

    let result = user_agent
        .process_transport_inbound(auth_request(ClientAuthPayload::AuthChallengeSolution(
            auth::AuthChallengeSolution {
                signature: serialized_signature,
            },
        )))
        .await
        .expect("Shouldn't fail to process message");

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
