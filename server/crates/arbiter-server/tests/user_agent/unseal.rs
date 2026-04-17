use arbiter_crypto::{
    authn,
    safecell::{SafeCell, SafeCellHandle as _},
};
use arbiter_server::{
    actors::{
        GlobalActors,
        vault::{Bootstrap, Seal},
    },
    db,
    peers::user_agent::{
        Credentials,
        vault_gate::{
            Error as VaultGateError, HandleHandshake, HandleUnsealEncryptedKey, VaultGate,
        },
    },
};

use chacha20poly1305::{AeadInPlace, XChaCha20Poly1305, XNonce, aead::KeyInit};
use kameo::actor::Spawn as _;
use tokio::sync::oneshot;
use x25519_dalek::{EphemeralSecret, PublicKey};

async fn setup_sealed_gate(
    seal_key: &[u8],
) -> (
    db::DatabasePool,
    kameo::actor::ActorRef<VaultGate>,
    oneshot::Receiver<Result<(), VaultGateError>>,
) {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();

    actors
        .vault
        .ask(Bootstrap {
            seal_key_raw: SafeCell::new(seal_key.to_vec()),
        })
        .await
        .unwrap();
    actors.vault.ask(Seal).await.unwrap();

    let (promotion_tx, promotion_rx) = oneshot::channel();
    let pubkey = authn::SigningKey::generate().public_key();
    let auth_creds = Credentials { id: 1, pubkey };
    let gate = VaultGate::spawn(VaultGate::new(auth_creds, actors, db.clone(), promotion_tx));

    (db, gate, promotion_rx)
}

async fn client_dh_encrypt(
    gate: &kameo::actor::ActorRef<VaultGate>,
    key_to_send: &[u8],
) -> HandleUnsealEncryptedKey {
    let client_secret = EphemeralSecret::random();
    let client_public = PublicKey::from(&client_secret);

    let response = gate
        .ask(HandleHandshake {
            client_pubkey: client_public,
        })
        .await
        .unwrap();

    let server_pubkey = response.server_pubkey;

    let shared_secret = client_secret.diffie_hellman(&server_pubkey);
    let cipher = XChaCha20Poly1305::new(shared_secret.as_bytes().into());
    let nonce = XNonce::from([0u8; 24]);
    let associated_data = b"unseal";
    let mut ciphertext = key_to_send.to_vec();
    cipher
        .encrypt_in_place(&nonce, associated_data, &mut ciphertext)
        .unwrap();

    HandleUnsealEncryptedKey {
        nonce: nonce.to_vec(),
        ciphertext,
        associated_data: associated_data.to_vec(),
    }
}

#[tokio::test]
#[test_log::test]
pub async fn test_unseal_success() {
    let seal_key = b"test-seal-key";
    let (_db, gate, _promotion_rx) = setup_sealed_gate(seal_key).await;

    let encrypted_key = client_dh_encrypt(&gate, seal_key).await;

    let response = gate.ask(encrypted_key).await;
    assert!(matches!(response, Ok(())));
}

#[tokio::test]
#[test_log::test]
pub async fn test_unseal_wrong_seal_key() {
    let (_db, gate, _promotion_rx) = setup_sealed_gate(b"correct-key").await;

    let encrypted_key = client_dh_encrypt(&gate, b"wrong-key").await;

    let response = gate.ask(encrypted_key).await;
    assert!(matches!(
        response,
        Err(kameo::error::SendError::HandlerError(
            VaultGateError::InvalidKey
        ))
    ));
}

#[tokio::test]
#[test_log::test]
pub async fn test_unseal_corrupted_ciphertext() {
    let (_db, gate, _promotion_rx) = setup_sealed_gate(b"test-key").await;

    let client_secret = EphemeralSecret::random();
    let client_public = PublicKey::from(&client_secret);

    gate.ask(HandleHandshake {
        client_pubkey: client_public,
    })
    .await
    .unwrap();

    let response = gate
        .ask(HandleUnsealEncryptedKey {
            nonce: vec![0u8; 24],
            ciphertext: vec![0u8; 32],
            associated_data: vec![],
        })
        .await;

    assert!(matches!(
        response,
        Err(kameo::error::SendError::HandlerError(
            VaultGateError::InvalidKey
        ))
    ));
}

#[tokio::test]
#[test_log::test]
pub async fn test_unseal_retry_after_invalid_key() {
    let seal_key = b"real-seal-key";
    let (_db, gate, _promotion_rx) = setup_sealed_gate(seal_key).await;

    {
        let encrypted_key = client_dh_encrypt(&gate, b"wrong-key").await;

        let response = gate.ask(encrypted_key).await;
        assert!(matches!(
            response,
            Err(kameo::error::SendError::HandlerError(
                VaultGateError::InvalidKey
            ))
        ));
    }

    {
        let encrypted_key = client_dh_encrypt(&gate, seal_key).await;

        let response = gate.ask(encrypted_key).await;
        assert!(matches!(response, Ok(())));
    }
}
