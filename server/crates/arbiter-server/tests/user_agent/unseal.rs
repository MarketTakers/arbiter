use arbiter_server::{
    actors::{
        GlobalActors,
        keyholder::{Bootstrap, Seal},
        user_agent::{Request, Response, UnsealError, session::UserAgentSession},
    },
    db,
};
use chacha20poly1305::{AeadInPlace, XChaCha20Poly1305, XNonce, aead::KeyInit};
use memsafe::MemSafe;
use x25519_dalek::{EphemeralSecret, PublicKey};

async fn setup_sealed_user_agent(seal_key: &[u8]) -> (db::DatabasePool, UserAgentSession) {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();

    actors
        .key_holder
        .ask(Bootstrap {
            seal_key_raw: MemSafe::new(seal_key.to_vec()).unwrap(),
        })
        .await
        .unwrap();
    actors.key_holder.ask(Seal).await.unwrap();

    let session = UserAgentSession::new_test(db.clone(), actors);

    (db, session)
}

async fn client_dh_encrypt(user_agent: &mut UserAgentSession, key_to_send: &[u8]) -> Request {
    let client_secret = EphemeralSecret::random();
    let client_public = PublicKey::from(&client_secret);

    let response = user_agent
        .process_transport_inbound(Request::UnsealStart {
            client_pubkey: client_public,
        })
        .await
        .unwrap();

    let server_pubkey = match response {
        Response::UnsealStartResponse { server_pubkey } => server_pubkey,
        other => panic!("Expected UnsealStartResponse, got {other:?}"),
    };

    let shared_secret = client_secret.diffie_hellman(&server_pubkey);
    let cipher = XChaCha20Poly1305::new(shared_secret.as_bytes().into());
    let nonce = XNonce::from([0u8; 24]);
    let associated_data = b"unseal";
    let mut ciphertext = key_to_send.to_vec();
    cipher
        .encrypt_in_place(&nonce, associated_data, &mut ciphertext)
        .unwrap();

    Request::UnsealEncryptedKey {
        nonce: nonce.to_vec(),
        ciphertext,
        associated_data: associated_data.to_vec(),
    }
}

#[tokio::test]
#[test_log::test]
pub async fn test_unseal_success() {
    let seal_key = b"test-seal-key";
    let (_db, mut user_agent) = setup_sealed_user_agent(seal_key).await;

    let encrypted_key = client_dh_encrypt(&mut user_agent, seal_key).await;

    let response = user_agent
        .process_transport_inbound(encrypted_key)
        .await
        .unwrap();

    assert!(matches!(response, Response::UnsealResult(Ok(()))));
}

#[tokio::test]
#[test_log::test]
pub async fn test_unseal_wrong_seal_key() {
    let (_db, mut user_agent) = setup_sealed_user_agent(b"correct-key").await;

    let encrypted_key = client_dh_encrypt(&mut user_agent, b"wrong-key").await;

    let response = user_agent
        .process_transport_inbound(encrypted_key)
        .await
        .unwrap();

    assert!(matches!(
        response,
        Response::UnsealResult(Err(UnsealError::InvalidKey))
    ));
}

#[tokio::test]
#[test_log::test]
pub async fn test_unseal_corrupted_ciphertext() {
    let (_db, mut user_agent) = setup_sealed_user_agent(b"test-key").await;

    let client_secret = EphemeralSecret::random();
    let client_public = PublicKey::from(&client_secret);

    user_agent
        .process_transport_inbound(Request::UnsealStart {
            client_pubkey: client_public,
        })
        .await
        .unwrap();

    let response = user_agent
        .process_transport_inbound(Request::UnsealEncryptedKey {
            nonce: vec![0u8; 24],
            ciphertext: vec![0u8; 32],
            associated_data: vec![],
        })
        .await
        .unwrap();

    assert!(matches!(
        response,
        Response::UnsealResult(Err(UnsealError::InvalidKey))
    ));
}

#[tokio::test]
#[test_log::test]
pub async fn test_unseal_retry_after_invalid_key() {
    let seal_key = b"real-seal-key";
    let (_db, mut user_agent) = setup_sealed_user_agent(seal_key).await;

    {
        let encrypted_key = client_dh_encrypt(&mut user_agent, b"wrong-key").await;

        let response = user_agent
            .process_transport_inbound(encrypted_key)
            .await
            .unwrap();

        assert!(matches!(
            response,
            Response::UnsealResult(Err(UnsealError::InvalidKey))
        ));
    }

    {
        let encrypted_key = client_dh_encrypt(&mut user_agent, seal_key).await;

        let response = user_agent
            .process_transport_inbound(encrypted_key)
            .await
            .unwrap();

        assert!(matches!(response, Response::UnsealResult(Ok(()))));
    }
}
