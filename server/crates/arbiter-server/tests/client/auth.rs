use arbiter_proto::proto::client::{
    AuthChallengeRequest, AuthChallengeSolution, AuthOk, ClientRequest, ClientResponse,
    client_request::Payload as ClientRequestPayload,
    client_response::Payload as ClientResponsePayload,
};
use arbiter_server::{
    actors::client::{ClientActor, ClientError},
    db::{self, schema},
};
use diesel::{ExpressionMethods as _, insert_into};
use diesel_async::RunQueryDsl;
use ed25519_dalek::Signer as _;

#[tokio::test]
#[test_log::test]
pub async fn test_unregistered_pubkey_rejected() {
    let db = db::create_test_pool().await;

    let mut client = ClientActor::new_manual(db.clone());

    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    let result = client
        .process_transport_inbound(ClientRequest {
            payload: Some(ClientRequestPayload::AuthChallengeRequest(
                AuthChallengeRequest {
                    pubkey: pubkey_bytes,
                },
            )),
        })
        .await;

    match result {
        Err(err) => {
            assert_eq!(err, ClientError::PublicKeyNotRegistered);
        }
        Ok(_) => {
            panic!("Expected error due to unregistered pubkey, but got success");
        }
    }
}

#[tokio::test]
#[test_log::test]
pub async fn test_challenge_auth() {
    let db = db::create_test_pool().await;

    let mut client = ClientActor::new_manual(db.clone());

    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    {
        let mut conn = db.get().await.unwrap();
        insert_into(schema::program_client::table)
            .values(schema::program_client::public_key.eq(pubkey_bytes.clone()))
            .execute(&mut conn)
            .await
            .unwrap();
    }

    let result = client
        .process_transport_inbound(ClientRequest {
            payload: Some(ClientRequestPayload::AuthChallengeRequest(
                AuthChallengeRequest {
                    pubkey: pubkey_bytes,
                },
            )),
        })
        .await
        .expect("Shouldn't fail to process message");

    let ClientResponse {
        payload: Some(ClientResponsePayload::AuthChallenge(challenge)),
    } = result
    else {
        panic!("Expected auth challenge response, got {result:?}");
    };

    let formatted_challenge = arbiter_proto::format_challenge(challenge.nonce, &challenge.pubkey);
    let signature = new_key.sign(&formatted_challenge);
    let serialized_signature = signature.to_bytes().to_vec();

    let result = client
        .process_transport_inbound(ClientRequest {
            payload: Some(ClientRequestPayload::AuthChallengeSolution(
                AuthChallengeSolution {
                    signature: serialized_signature,
                },
            )),
        })
        .await
        .expect("Shouldn't fail to process message");

    assert_eq!(
        result,
        ClientResponse {
            payload: Some(ClientResponsePayload::AuthOk(AuthOk {})),
        }
    );
}
