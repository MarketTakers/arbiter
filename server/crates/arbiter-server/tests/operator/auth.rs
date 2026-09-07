use super::common::{ChannelTransport, bootstrapped_vault, eventually, spawn_actors};
use arbiter_crypto::authn::{self, AuthChallenge, SigningContext};
use arbiter_proto::transport::{Error as TransportError, Receiver, Sender};
use arbiter_server::{
    actors::{
        bootstrap::GetToken,
        vault::{self, Bootstrap},
    },
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
        .sign_deterministic(&challenge, SigningContext::Operator.as_bytes())
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
    let actors = spawn_actors(db.clone()).await;
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

/// A multi-operator committee must all register with the same bootstrap token before bootstrap
/// completes, so verifying the token must not consume it. This is the reachability bug fixed by
/// replacing `consume_token` with `verify_token`.
#[tokio::test]
#[test_log::test]
pub async fn bootstrap_token_registers_every_committee_member() {
    let db = db::create_test_pool().await;
    let actors = spawn_actors(db.clone()).await;
    let token = actors.bootstrapper.ask(GetToken).await.unwrap().unwrap();

    for _ in 0..2 {
        let (mut server_transport, mut test_transport) = ChannelTransport::new();
        let db_for_task = db.clone();
        let actors_for_task = actors.clone();
        let task = tokio::spawn(async move {
            let mut props = OperatorConnection::new(db_for_task, actors_for_task);
            auth::authenticate(&mut props, &mut server_transport).await
        });

        let new_key = MlDsa87::key_gen(&mut rand::rng());
        test_transport
            .send(auth::Inbound::AuthChallengeRequest {
                pubkey: verifying_key(&new_key).into(),
                bootstrap_token: Some(token.clone()),
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
    }

    let mut conn = db.get().await.unwrap();
    let registered: i64 = schema::operator_identity::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(registered, 2);

    // Bootstrap has not completed: the token must still be valid.
    assert_eq!(
        actors.bootstrapper.ask(GetToken).await.unwrap(),
        Some(token)
    );
}

/// `GlobalActors` must subscribe `Bootstrapper` to `events::Bootstrapped` on the message bus.
/// Without that one registration the token stays valid forever in production, which is the
/// defect this task exists to fix, and no other test notices: the test below drives the event
/// handler directly, and the `challenge_auth` family never re-reads the token after
/// bootstrapping. This one goes the whole way round -- real `Vault::bootstrap`, real bus --
/// and waits for the effect rather than reading straight after the call, because `Publish`
/// only enqueues to the bus's mailbox.
#[tokio::test]
#[test_log::test]
pub async fn bootstrapped_event_retires_the_token_through_the_message_bus() {
    let db = db::create_test_pool().await;
    let actors = spawn_actors(db.clone()).await;

    assert!(
        actors.bootstrapper.ask(GetToken).await.unwrap().is_some(),
        "the token must exist before the vault is bootstrapped"
    );

    actors
        .vault
        .ask(Bootstrap {
            seal_key: arbiter_server::crypto::KeyCell::from([0u8; 32]),
        })
        .await
        .unwrap();

    eventually("the bootstrap token to be retired", || async {
        actors
            .bootstrapper
            .ask(GetToken)
            .await
            .unwrap()
            .is_none()
            .then_some(())
    })
    .await;
}

/// Once the vault reports `Bootstrapped`, the token is retired: further registrations must be
/// rejected even with a token that verified successfully moments earlier.
#[tokio::test]
#[test_log::test]
pub async fn bootstrap_token_rejected_after_bootstrapped_event() {
    let db = db::create_test_pool().await;
    let actors = spawn_actors(db.clone()).await;
    let token = actors.bootstrapper.ask(GetToken).await.unwrap().unwrap();

    // Drive the Bootstrapper's own event handler directly rather than through
    // `actors.vault.ask(Bootstrap { .. })` + the message bus: bus delivery is fire-and-forget,
    // so asserting on it would be racy. The handler under test is the same either way.
    actors
        .bootstrapper
        .ask(vault::events::Bootstrapped)
        .await
        .unwrap();
    assert!(actors.bootstrapper.ask(GetToken).await.unwrap().is_none());

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

/// `register_key`'s database gate must refuse a registration once `arbiter_settings.root_key_id`
/// is set, even when `Bootstrapper`'s own in-memory token has not yet been retired -- exactly
/// the two-mailbox-hop window between `Vault::bootstrap`'s commit and the `Bootstrapped` event
/// reaching `Bootstrapper` in production. "database bootstrapped, Bootstrapper not yet notified"
/// is reproduced deterministically by bootstrapping a throwaway `Vault` wired to its own message
/// bus: it commits `root_key_id` in the same database without ever publishing to the bus
/// `actors.bootstrapper` is registered on, so `actors.bootstrapper`'s token is left untouched.
#[tokio::test]
#[test_log::test]
pub async fn bootstrap_token_rejected_once_the_database_is_bootstrapped() {
    let db = db::create_test_pool().await;
    let actors = spawn_actors(db.clone()).await;
    let token = actors.bootstrapper.ask(GetToken).await.unwrap().unwrap();

    bootstrapped_vault(&db).await;

    // From Bootstrapper's point of view the token still verifies: it never received an event.
    assert_eq!(
        actors.bootstrapper.ask(GetToken).await.unwrap(),
        Some(token.clone())
    );

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

    // The refusal has to reach the peer, not just the task's return value: a registration
    // refused after the database was bootstrapped must look like the refusal of a token that
    // never verified, rather than a handshake that stops with nothing on the wire.
    let refusal = test_transport
        .recv()
        .await
        .expect("the refusal must be sent to the peer");
    assert!(matches!(refusal, Err(auth::Error::InvalidBootstrapToken)));

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
pub async fn bootstrap_invalid_token_auth() {
    let db = db::create_test_pool().await;
    let actors = spawn_actors(db.clone()).await;

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

    // The reference behaviour the refusal above has to match: a token that never verified is
    // reported to the peer. Pinned here so the two refusal paths cannot drift apart again.
    let refusal = test_transport
        .recv()
        .await
        .expect("the refusal must be sent to the peer");
    assert!(matches!(refusal, Err(auth::Error::InvalidBootstrapToken)));

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
    let actors = spawn_actors(db.clone()).await;
    actors
        .vault
        .ask(Bootstrap {
            seal_key: arbiter_server::crypto::KeyCell::from([0u8; 32]),
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
    let actors = spawn_actors(db.clone()).await;

    actors
        .vault
        .ask(Bootstrap {
            seal_key: arbiter_server::crypto::KeyCell::from([0u8; 32]),
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
    let actors = spawn_actors(db.clone()).await;
    actors
        .vault
        .ask(Bootstrap {
            seal_key: arbiter_server::crypto::KeyCell::from([0u8; 32]),
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
