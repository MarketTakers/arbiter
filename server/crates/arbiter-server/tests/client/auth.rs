use arbiter_proto::transport::Bi;
use arbiter_server::actors::GlobalActors;
use arbiter_server::{
    actors::client::{ClientConnection, Request, Response, connect_client},
    db::{self, schema},
};
use diesel::{ExpressionMethods as _, insert_into};
use diesel_async::RunQueryDsl;
use ed25519_dalek::Signer as _;

use super::common::ChannelTransport;

#[tokio::test]
#[test_log::test]
pub async fn test_unregistered_pubkey_rejected() {
    let db = db::create_test_pool().await;

    let (server_transport, mut test_transport) = ChannelTransport::new();
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    let props = ClientConnection::new(db.clone(), Box::new(server_transport), actors);
    let task = tokio::spawn(connect_client(props));

    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    test_transport
        .send(Request::AuthChallengeRequest {
            pubkey: pubkey_bytes,
        })
        .await
        .unwrap();

    // Auth fails, connect_client returns, transport drops
    task.await.unwrap();
}

#[tokio::test]
#[test_log::test]
pub async fn test_challenge_auth() {
    let db = db::create_test_pool().await;

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

    let (server_transport, mut test_transport) = ChannelTransport::new();
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();

    let props = ClientConnection::new(db.clone(), Box::new(server_transport), actors);
    let task = tokio::spawn(connect_client(props));

    // Send challenge request
    test_transport
        .send(Request::AuthChallengeRequest {
            pubkey: pubkey_bytes,
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
            Response::AuthChallenge { pubkey, nonce } => (pubkey, nonce),
            other => panic!("Expected AuthChallenge, got {other:?}"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    };

    // Sign the challenge and send solution
    let formatted_challenge = arbiter_proto::format_challenge(challenge.1, &challenge.0);
    let signature = new_key.sign(&formatted_challenge);

    test_transport
        .send(Request::AuthChallengeSolution {
            signature: signature.to_bytes().to_vec(),
        })
        .await
        .unwrap();

    // Auth completes, session spawned
    task.await.unwrap();
}
