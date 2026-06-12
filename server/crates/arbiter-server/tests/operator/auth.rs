use super::common::ChannelTransport;
use arbiter_crypto::{
    authn::{self, AuthChallenge, OPERATOR_CONTEXT},
    safecell::{SafeCell, SafeCellHandle as _},
};
use arbiter_proto::transport::{Error as TransportError, Receiver, Sender};
use arbiter_server::{
    actors::{GlobalActors, bootstrap::GetToken, vault::Bootstrap},
    crypto::integrity,
    db::{self, schema},
    peers::operator::{self, Credentials, OperatorConnection, auth, vault_gate},
};

use async_trait::async_trait;
use diesel::{ExpressionMethods as _, QueryDsl, insert_into};
use diesel_async::RunQueryDsl;
use ml_dsa::{KeyGen, MlDsa87, SigningKey, VerifyingKey, signature::Keypair};
use tokio::sync::mpsc;

fn verifying_key(key: &SigningKey<MlDsa87>) -> VerifyingKey<MlDsa87> {
    <SigningKey<MlDsa87> as Keypair>::verifying_key(key)
}

fn sign_operator_challenge(
    key: &SigningKey<MlDsa87>,
    challenge: &AuthChallenge,
) -> authn::Signature {
    let challenge = challenge.format();
    key.signing_key()
        .sign_deterministic(&challenge, OPERATOR_CONTEXT)
        .unwrap()
        .into()
}

fn tamper_challenge(challenge: &AuthChallenge) -> AuthChallenge {
    let mut challenge = challenge.clone();
    challenge.nonce[0] ^= 1;
    challenge
}

struct NullOobSender;

#[async_trait]
impl Sender<operator::OutOfBand> for NullOobSender {
    async fn send(&mut self, _item: operator::OutOfBand) -> Result<(), TransportError> {
        Ok(())
    }
}

struct StartServerTransport {
    auth_rx: mpsc::Receiver<auth::Inbound>,
    auth_tx: mpsc::Sender<Result<auth::Outbound, auth::Error>>,
    vault_rx: mpsc::Receiver<vault_gate::Inbound>,
    vault_tx: mpsc::Sender<Result<vault_gate::Outbound, vault_gate::Error>>,
}

struct StartTestTransport {
    auth_rx: mpsc::Receiver<Result<auth::Outbound, auth::Error>>,
    auth_tx: mpsc::Sender<auth::Inbound>,
}

fn start_transport_pair() -> (StartServerTransport, StartTestTransport) {
    let (auth_in_tx, auth_in_rx) = mpsc::channel(10);
    let (auth_out_tx, auth_out_rx) = mpsc::channel(10);
    let (_vault_in_tx, vault_in_rx) = mpsc::channel(10);
    let (vault_out_tx, _vault_out_rx) = mpsc::channel(10);

    (
        StartServerTransport {
            auth_rx: auth_in_rx,
            auth_tx: auth_out_tx,
            vault_rx: vault_in_rx,
            vault_tx: vault_out_tx,
        },
        StartTestTransport {
            auth_rx: auth_out_rx,
            auth_tx: auth_in_tx,
        },
    )
}

#[async_trait]
impl Receiver<auth::Inbound> for StartServerTransport {
    async fn recv(&mut self) -> Option<auth::Inbound> {
        self.auth_rx.recv().await
    }
}

#[async_trait]
impl Sender<Result<auth::Outbound, auth::Error>> for StartServerTransport {
    async fn send(
        &mut self,
        item: Result<auth::Outbound, auth::Error>,
    ) -> Result<(), TransportError> {
        self.auth_tx
            .send(item)
            .await
            .map_err(|_| TransportError::ChannelClosed)
    }
}

impl arbiter_proto::transport::Bi<auth::Inbound, Result<auth::Outbound, auth::Error>>
    for StartServerTransport
{
}

#[async_trait]
impl Receiver<vault_gate::Inbound> for StartServerTransport {
    async fn recv(&mut self) -> Option<vault_gate::Inbound> {
        self.vault_rx.recv().await
    }
}

#[async_trait]
impl Sender<Result<vault_gate::Outbound, vault_gate::Error>> for StartServerTransport {
    async fn send(
        &mut self,
        item: Result<vault_gate::Outbound, vault_gate::Error>,
    ) -> Result<(), TransportError> {
        self.vault_tx
            .send(item)
            .await
            .map_err(|_| TransportError::ChannelClosed)
    }
}

impl
    arbiter_proto::transport::Bi<
        vault_gate::Inbound,
        Result<vault_gate::Outbound, vault_gate::Error>,
    > for StartServerTransport
{
}

#[async_trait]
impl Receiver<Result<auth::Outbound, auth::Error>> for StartTestTransport {
    async fn recv(&mut self) -> Option<Result<auth::Outbound, auth::Error>> {
        self.auth_rx.recv().await
    }
}

#[async_trait]
impl Sender<auth::Inbound> for StartTestTransport {
    async fn send(&mut self, item: auth::Inbound) -> Result<(), TransportError> {
        self.auth_tx
            .send(item)
            .await
            .map_err(|_| TransportError::ChannelClosed)
    }
}

#[tokio::test]
#[test_log::test]
pub async fn bootstrap_token_auth() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap {
            seal_key_raw: SafeCell::new([0u8; 32].to_vec()),
        })
        .await
        .unwrap();
    let token = actors.bootstrapper.ask(GetToken).await.unwrap().unwrap();

    let (mut server_transport, mut test_transport) = ChannelTransport::new();
    let db_for_task = db.clone();
    let task = tokio::spawn(async move {
        let mut props = OperatorConnection::new(db_for_task, actors);
        auth::authenticate(&mut props, &mut server_transport).await
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
        .expect("should receive challenge");
    let challenge = match response {
        Ok(auth::Outbound::AuthChallenge { challenge }) => challenge,
        other => panic!("Expected AuthChallenge, got {other:?}"),
    };

    let signature = sign_operator_challenge(&new_key, &challenge);

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
    assert!(matches!(response, Ok(auth::Outbound::AuthSuccess)));

    task.await.unwrap().unwrap();

    let mut conn = db.get().await.unwrap();
    let stored_pubkey: Vec<u8> = schema::operator_identity::table
        .select(schema::operator_identity::public_key)
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

    let (mut server_transport, mut test_transport) = ChannelTransport::new();
    let db_for_task = db.clone();
    let task = tokio::spawn(async move {
        let mut props = OperatorConnection::new(db_for_task, actors);
        auth::authenticate(&mut props, &mut server_transport).await
    });

    let new_key = MlDsa87::key_gen(&mut rand::rng());
    test_transport
        .send(auth::Inbound::AuthChallengeRequest {
            pubkey: verifying_key(&new_key).into(),
            bootstrap_token: Some("invalid_token".to_owned()),
        })
        .await
        .unwrap();

    let response = test_transport
        .recv()
        .await
        .expect("should receive challenge");
    let challenge = match response {
        Ok(auth::Outbound::AuthChallenge { challenge }) => challenge,
        other => panic!("Expected AuthChallenge, got {other:?}"),
    };

    let signature = sign_operator_challenge(&new_key, &challenge);
    test_transport
        .send(auth::Inbound::AuthChallengeSolution {
            signature: signature.to_bytes(),
        })
        .await
        .unwrap();

    assert!(matches!(
        task.await.unwrap(),
        Err(auth::Error::InvalidBootstrapToken)
    ));

    let mut conn = db.get().await.unwrap();
    let count: i64 = schema::operator_identity::table
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
        .vault
        .ask(Bootstrap {
            seal_key_raw: SafeCell::new([0u8; 32].to_vec()),
        })
        .await
        .unwrap();

    let new_key = MlDsa87::key_gen(&mut rand::rng());
    let pubkey_bytes = authn::PublicKey::from(verifying_key(&new_key)).to_bytes();

    {
        let mut conn = db.get().await.unwrap();
        let id: i32 = insert_into(schema::operator_identity::table)
            .values((schema::operator_identity::public_key.eq(pubkey_bytes.clone()),))
            .returning(schema::operator_identity::id)
            .get_result(&mut conn)
            .await
            .unwrap();
        integrity::sign_entity(
            &mut conn,
            &actors.vault,
            &Credentials {
                id,
                pubkey: verifying_key(&new_key).into(),
            },
            id,
        )
        .await
        .unwrap();
    }

    let (mut server_transport, mut test_transport) = ChannelTransport::new();
    let db_for_task = db.clone();
    let task = tokio::spawn(async move {
        let mut props = OperatorConnection::new(db_for_task, actors);
        auth::authenticate(&mut props, &mut server_transport).await
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
            auth::Outbound::AuthChallenge { challenge } => challenge,
            auth::Outbound::AuthSuccess => panic!("Expected AuthChallenge, got AuthSuccess"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    };

    let signature = sign_operator_challenge(&new_key, &challenge);

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
        .vault
        .ask(Bootstrap {
            seal_key_raw: SafeCell::new([0u8; 32].to_vec()),
        })
        .await
        .unwrap();

    let new_key = MlDsa87::key_gen(&mut rand::rng());
    let pubkey_bytes = authn::PublicKey::from(verifying_key(&new_key)).to_bytes();

    {
        let mut conn = db.get().await.unwrap();
        insert_into(schema::operator_identity::table)
            .values((schema::operator_identity::public_key.eq(pubkey_bytes.clone()),))
            .execute(&mut conn)
            .await
            .unwrap();
    }

    let (server_transport, mut test_transport) = start_transport_pair();
    let db_for_task = db.clone();
    let task = tokio::spawn(async move {
        let mut props = OperatorConnection::new(db_for_task, actors);
        operator::start(&mut props, server_transport, Box::new(NullOobSender)).await
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
            auth::Outbound::AuthChallenge { challenge } => challenge,
            other => panic!("Expected AuthChallenge, got {other:?}"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    };

    let signature = sign_operator_challenge(&new_key, &challenge);

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
    assert!(matches!(response, Ok(auth::Outbound::AuthSuccess)));

    assert!(matches!(
        task.await.unwrap(),
        Err(operator::Error::Internal(_))
    ));
}

#[tokio::test]
#[test_log::test]
pub async fn challenge_auth_rejects_invalid_signature() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap {
            seal_key_raw: SafeCell::new([0u8; 32].to_vec()),
        })
        .await
        .unwrap();

    let new_key = MlDsa87::key_gen(&mut rand::rng());
    let pubkey_bytes = authn::PublicKey::from(verifying_key(&new_key)).to_bytes();

    {
        let mut conn = db.get().await.unwrap();
        let id: i32 = insert_into(schema::operator_identity::table)
            .values((schema::operator_identity::public_key.eq(pubkey_bytes.clone()),))
            .returning(schema::operator_identity::id)
            .get_result(&mut conn)
            .await
            .unwrap();
        integrity::sign_entity(
            &mut conn,
            &actors.vault,
            &Credentials {
                id,
                pubkey: verifying_key(&new_key).into(),
            },
            id,
        )
        .await
        .unwrap();
    }

    let (mut server_transport, mut test_transport) = ChannelTransport::new();
    let db_for_task = db.clone();
    let task = tokio::spawn(async move {
        let mut props = OperatorConnection::new(db_for_task, actors);
        auth::authenticate(&mut props, &mut server_transport).await
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
            auth::Outbound::AuthChallenge { challenge } => challenge,
            auth::Outbound::AuthSuccess => panic!("Expected AuthChallenge, got AuthSuccess"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    };

    let signature = sign_operator_challenge(&new_key, &tamper_challenge(&challenge));

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
