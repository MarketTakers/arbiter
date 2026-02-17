use arbiter_proto::proto::{
    UnsealEncryptedKey, UnsealResult, UnsealStart, auth::AuthChallengeRequest,
    user_agent_response::Payload as UserAgentResponsePayload,
};
use arbiter_server::{
    actors::{
        GlobalActors,
        bootstrap::GetToken,
        keyholder::{Bootstrap, Seal},
        user_agent::{
            HandleAuthChallengeRequest, HandleUnsealEncryptedKey, HandleUnsealRequest,
            UserAgentActor,
        },
    },
    db,
};
use chacha20poly1305::{AeadInPlace, XChaCha20Poly1305, XNonce, aead::KeyInit};
use kameo::actor::{ActorRef, Spawn};
use memsafe::MemSafe;
use x25519_dalek::{EphemeralSecret, PublicKey};

async fn setup_authenticated_user_agent(
    seal_key: &[u8],
) -> (arbiter_server::db::DatabasePool, ActorRef<UserAgentActor>) {
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

    let user_agent =
        UserAgentActor::new_manual(db.clone(), actors.clone(), tokio::sync::mpsc::channel(1).0);
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
    let server_public = PublicKey::from(<[u8; 32]>::try_from(server_pubkey.as_slice()).unwrap());

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

    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    let user_agent =
        UserAgentActor::new_manual(db.clone(), actors, tokio::sync::mpsc::channel(1).0);
    let user_agent_ref = UserAgentActor::spawn(user_agent);

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
