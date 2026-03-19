use alloy::{
    consensus::TxEip1559,
    primitives::{Address, Bytes, TxKind, U256},
    rlp::Encodable,
};
use arbiter_proto::proto::{
    client::{
        AuthChallengeRequest, AuthChallengeSolution, ClientRequest,
        client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
    },
    evm::EvmSignTransactionRequest,
};
use arbiter_proto::transport::Bi;
use arbiter_server::actors::GlobalActors;
use arbiter_server::{
    actors::client::{ClientConnection, connect_client},
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
        .send(ClientRequest {
            payload: Some(ClientRequestPayload::AuthChallengeRequest(
                AuthChallengeRequest {
                    pubkey: pubkey_bytes,
                },
            )),
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
        .send(ClientRequest {
            payload: Some(ClientRequestPayload::AuthChallengeRequest(
                AuthChallengeRequest {
                    pubkey: pubkey_bytes,
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
            Some(ClientResponsePayload::AuthChallenge(c)) => c,
            other => panic!("Expected AuthChallenge, got {other:?}"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    };

    // Sign the challenge and send solution
    let formatted_challenge = arbiter_proto::format_challenge(challenge.nonce, &challenge.pubkey);
    let signature = new_key.sign(&formatted_challenge);

    test_transport
        .send(ClientRequest {
            payload: Some(ClientRequestPayload::AuthChallengeSolution(
                AuthChallengeSolution {
                    signature: signature.to_bytes().to_vec(),
                },
            )),
        })
        .await
        .unwrap();

    let response = test_transport.recv().await.expect("should receive auth ok");
    match response {
        Ok(resp) => match resp.payload {
            Some(ClientResponsePayload::AuthOk(_)) => {}
            other => panic!("Expected AuthOk, got {other:?}"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    }

    // Auth completes, session spawned
    task.await.unwrap();
}

#[tokio::test]
#[test_log::test]
pub async fn test_evm_sign_request_payload_is_handled() {
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

    test_transport
        .send(ClientRequest {
            payload: Some(ClientRequestPayload::AuthChallengeRequest(
                AuthChallengeRequest {
                    pubkey: pubkey_bytes,
                },
            )),
        })
        .await
        .unwrap();

    let response = test_transport
        .recv()
        .await
        .expect("should receive challenge");
    let challenge = match response {
        Ok(resp) => match resp.payload {
            Some(ClientResponsePayload::AuthChallenge(c)) => c,
            other => panic!("Expected AuthChallenge, got {other:?}"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    };

    let formatted_challenge = arbiter_proto::format_challenge(challenge.nonce, &challenge.pubkey);
    let signature = new_key.sign(&formatted_challenge);

    test_transport
        .send(ClientRequest {
            payload: Some(ClientRequestPayload::AuthChallengeSolution(
                AuthChallengeSolution {
                    signature: signature.to_bytes().to_vec(),
                },
            )),
        })
        .await
        .unwrap();

    let response = test_transport.recv().await.expect("should receive auth ok");
    match response {
        Ok(resp) => match resp.payload {
            Some(ClientResponsePayload::AuthOk(_)) => {}
            other => panic!("Expected AuthOk, got {other:?}"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    }

    task.await.unwrap();

    let tx = TxEip1559 {
        chain_id: 1,
        nonce: 0,
        gas_limit: 21_000,
        max_fee_per_gas: 1,
        max_priority_fee_per_gas: 1,
        to: TxKind::Call(Address::from_slice(&[0x11; 20])),
        value: U256::ZERO,
        input: Bytes::new(),
        access_list: Default::default(),
    };

    let mut rlp_transaction = Vec::new();
    tx.encode(&mut rlp_transaction);

    test_transport
        .send(ClientRequest {
            payload: Some(ClientRequestPayload::EvmSignTransaction(
                EvmSignTransactionRequest {
                    wallet_address: [0x22; 20].to_vec(),
                    rlp_transaction,
                },
            )),
        })
        .await
        .unwrap();

    let response = test_transport
        .recv()
        .await
        .expect("should receive sign response");

    match response {
        Ok(resp) => match resp.payload {
            Some(ClientResponsePayload::EvmSignTransaction(_)) => {}
            other => panic!("Expected EvmSignTransaction response, got {other:?}"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    }
}
