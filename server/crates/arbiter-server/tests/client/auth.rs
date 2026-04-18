use super::common::ChannelTransport;
use arbiter_crypto::{
    authn::{self, AuthChallenge, CLIENT_CONTEXT},
    safecell::{SafeCell, SafeCellHandle as _},
};
use arbiter_proto::{
    ClientMetadata,
    transport::{Receiver, Sender},
};
use arbiter_server::{
    actors::{GlobalActors, vault::Bootstrap},
    crypto::integrity,
    db::{self, schema},
    peers::client::{ClientConnection, ClientCredentials, auth, connect_client},
};

use diesel::{ExpressionMethods as _, NullableExpressionMethods as _, QueryDsl as _, insert_into};
use diesel_async::RunQueryDsl;
use ml_dsa::{KeyGen, MlDsa87, SigningKey, VerifyingKey, signature::Keypair as _};

fn metadata(name: &str, description: Option<&str>, version: Option<&str>) -> ClientMetadata {
    ClientMetadata {
        name: name.to_owned(),
        description: description.map(str::to_owned),
        version: version.map(str::to_owned),
    }
}

async fn insert_registered_client(
    db: &db::DatabasePool,
    actors: &GlobalActors,
    pubkey: VerifyingKey<MlDsa87>,
    metadata: &ClientMetadata,
) {
    use arbiter_server::db::schema::{client_metadata, program_client};

    let mut conn = db.get().await.unwrap();
    let metadata_id: i32 = insert_into(client_metadata::table)
        .values((
            client_metadata::name.eq(&metadata.name),
            client_metadata::description.eq(&metadata.description),
            client_metadata::version.eq(&metadata.version),
        ))
        .returning(client_metadata::id)
        .get_result(&mut conn)
        .await
        .unwrap();
    let client_id: i32 = insert_into(program_client::table)
        .values((
            program_client::public_key.eq(pubkey.encode().to_vec()),
            program_client::metadata_id.eq(metadata_id),
        ))
        .returning(program_client::id)
        .get_result(&mut conn)
        .await
        .unwrap();

    integrity::sign_entity(
        &mut conn,
        &actors.vault,
        &ClientCredentials {
            pubkey: pubkey.into(),
        },
        client_id,
    )
    .await
    .unwrap();
}

fn sign_client_challenge(key: &SigningKey<MlDsa87>, challenge: &AuthChallenge) -> authn::Signature {
    let challenge = challenge.format();
    key.signing_key()
        .sign_deterministic(&challenge, CLIENT_CONTEXT)
        .unwrap()
        .into()
}

async fn insert_bootstrap_sentinel_useragent(db: &db::DatabasePool) {
    let mut conn = db.get().await.unwrap();
    let sentinel_key = MlDsa87::key_gen(&mut rand::rng())
        .verifying_key()
        .encode()
        .to_vec();

    insert_into(schema::useragent_client::table)
        .values((schema::useragent_client::public_key.eq(sentinel_key),))
        .execute(&mut conn)
        .await
        .unwrap();
}

async fn spawn_test_actors(db: &db::DatabasePool) -> GlobalActors {
    insert_bootstrap_sentinel_useragent(db).await;

    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap {
            seal_key_raw: SafeCell::new(b"test-seal-key".to_vec()),
        })
        .await
        .unwrap();
    actors
}

#[tokio::test]
#[test_log::test]
async fn test_unregistered_pubkey_rejected() {
    let db = db::create_test_pool().await;

    let (server_transport, mut test_transport) = ChannelTransport::new();
    let actors = spawn_test_actors(&db).await;
    let props = ClientConnection::new(db.clone(), actors);
    let task = tokio::spawn(async move {
        let mut server_transport = server_transport;
        connect_client(props, &mut server_transport).await;
    });

    let new_key = MlDsa87::key_gen(&mut rand::rng());

    test_transport
        .send(auth::Inbound::AuthChallengeRequest {
            pubkey: new_key.verifying_key().into(),
            metadata: metadata("client", Some("desc"), Some("1.0.0")),
        })
        .await
        .unwrap();

    // Auth fails, connect_client returns, transport drops
    task.await.unwrap();
}

#[tokio::test]
#[test_log::test]
async fn test_challenge_auth() {
    let db = db::create_test_pool().await;
    let actors = spawn_test_actors(&db).await;

    let new_key = MlDsa87::key_gen(&mut rand::rng());

    insert_registered_client(
        &db,
        &actors,
        new_key.verifying_key(),
        &metadata("client", Some("desc"), Some("1.0.0")),
    )
    .await;

    let (server_transport, mut test_transport) = ChannelTransport::new();
    let props = ClientConnection::new(db.clone(), actors);
    let task = tokio::spawn(async move {
        let mut server_transport = server_transport;
        connect_client(props, &mut server_transport).await;
    });

    // Send challenge request
    test_transport
        .send(auth::Inbound::AuthChallengeRequest {
            pubkey: new_key.verifying_key().into(),
            metadata: metadata("client", Some("desc"), Some("1.0.0")),
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
            auth::Outbound::AuthChallenge { challenge } => challenge,
            other => panic!("Expected AuthChallenge, got {other:?}"),
        },
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    };

    // Sign the challenge and send solution
    let signature = sign_client_challenge(&new_key, &challenge);

    test_transport
        .send(auth::Inbound::AuthChallengeSolution { signature })
        .await
        .unwrap();

    let response = test_transport
        .recv()
        .await
        .expect("should receive auth success");
    match response {
        Ok(auth::Outbound::AuthSuccess) => {}
        Ok(other) => panic!("Expected AuthSuccess, got {other:?}"),
        Err(err) => panic!("Expected Ok response, got Err({err:?})"),
    }

    // Auth completes, session spawned
    task.await.unwrap();
}

#[tokio::test]
#[test_log::test]
async fn test_metadata_unchanged_does_not_append_history() {
    let db = db::create_test_pool().await;
    let actors = spawn_test_actors(&db).await;
    let new_key = MlDsa87::key_gen(&mut rand::rng());
    let requested = metadata("client", Some("desc"), Some("1.0.0"));

    insert_registered_client(&db, &actors, new_key.verifying_key(), &requested).await;

    let props = ClientConnection::new(db.clone(), actors);

    let (server_transport, mut test_transport) = ChannelTransport::new();
    let task = tokio::spawn(async move {
        let mut server_transport = server_transport;
        connect_client(props, &mut server_transport).await;
    });

    test_transport
        .send(auth::Inbound::AuthChallengeRequest {
            pubkey: new_key.verifying_key().into(),
            metadata: requested,
        })
        .await
        .unwrap();

    let response = test_transport.recv().await.unwrap().unwrap();
    let challenge = match response {
        auth::Outbound::AuthChallenge { challenge } => challenge,
        other => panic!("Expected AuthChallenge, got {other:?}"),
    };
    let signature = sign_client_challenge(&new_key, &challenge);
    test_transport
        .send(auth::Inbound::AuthChallengeSolution { signature })
        .await
        .unwrap();
    let _ = test_transport.recv().await.unwrap();
    task.await.unwrap();

    {
        use arbiter_server::db::schema::{client_metadata, client_metadata_history};
        let mut conn = db.get().await.unwrap();
        let metadata_count: i64 = client_metadata::table
            .count()
            .get_result(&mut conn)
            .await
            .unwrap();
        let history_count: i64 = client_metadata_history::table
            .count()
            .get_result(&mut conn)
            .await
            .unwrap();
        assert_eq!(metadata_count, 1);
        assert_eq!(history_count, 0);
    }
}

#[tokio::test]
#[test_log::test]
async fn test_metadata_change_appends_history_and_repoints_binding() {
    let db = db::create_test_pool().await;
    let actors = spawn_test_actors(&db).await;
    let new_key = MlDsa87::key_gen(&mut rand::rng());

    insert_registered_client(
        &db,
        &actors,
        new_key.verifying_key(),
        &metadata("client", Some("old"), Some("1.0.0")),
    )
    .await;

    let props = ClientConnection::new(db.clone(), actors);

    let (server_transport, mut test_transport) = ChannelTransport::new();
    let task = tokio::spawn(async move {
        let mut server_transport = server_transport;
        connect_client(props, &mut server_transport).await;
    });

    test_transport
        .send(auth::Inbound::AuthChallengeRequest {
            pubkey: new_key.verifying_key().into(),
            metadata: metadata("client", Some("new"), Some("2.0.0")),
        })
        .await
        .unwrap();

    let response = test_transport.recv().await.unwrap().unwrap();
    let challenge = match response {
        auth::Outbound::AuthChallenge { challenge } => challenge,
        other => panic!("Expected AuthChallenge, got {other:?}"),
    };
    let signature = sign_client_challenge(&new_key, &challenge);
    test_transport
        .send(auth::Inbound::AuthChallengeSolution { signature })
        .await
        .unwrap();
    let _ = test_transport.recv().await.unwrap();
    task.await.unwrap();

    {
        use arbiter_server::db::schema::{
            client_metadata, client_metadata_history, program_client,
        };
        let mut conn = db.get().await.unwrap();
        let metadata_count: i64 = client_metadata::table
            .count()
            .get_result(&mut conn)
            .await
            .unwrap();
        let history_count: i64 = client_metadata_history::table
            .count()
            .get_result(&mut conn)
            .await
            .unwrap();
        let metadata_id = program_client::table
            .select(program_client::metadata_id)
            .first::<i32>(&mut conn)
            .await
            .unwrap();
        let current = client_metadata::table
            .find(metadata_id)
            .select((
                client_metadata::name,
                client_metadata::description.nullable(),
                client_metadata::version.nullable(),
            ))
            .first::<(String, Option<String>, Option<String>)>(&mut conn)
            .await
            .unwrap();
        assert_eq!(metadata_count, 2);
        assert_eq!(history_count, 1);
        assert_eq!(
            current,
            (
                "client".to_owned(),
                Some("new".to_owned()),
                Some("2.0.0".to_owned())
            )
        );
    }
}

#[tokio::test]
#[test_log::test]
async fn test_challenge_auth_rejects_integrity_tag_mismatch() {
    let db = db::create_test_pool().await;
    let actors = spawn_test_actors(&db).await;

    let new_key = MlDsa87::key_gen(&mut rand::rng());
    let requested = metadata("client", Some("desc"), Some("1.0.0"));

    {
        use arbiter_server::db::schema::{client_metadata, program_client};
        let mut conn = db.get().await.unwrap();
        let metadata_id: i32 = insert_into(client_metadata::table)
            .values((
                client_metadata::name.eq(&requested.name),
                client_metadata::description.eq(&requested.description),
                client_metadata::version.eq(&requested.version),
            ))
            .returning(client_metadata::id)
            .get_result(&mut conn)
            .await
            .unwrap();
        insert_into(program_client::table)
            .values((
                program_client::public_key.eq(new_key.verifying_key().encode().to_vec()),
                program_client::metadata_id.eq(metadata_id),
            ))
            .execute(&mut conn)
            .await
            .unwrap();
    }

    let (server_transport, mut test_transport) = ChannelTransport::new();
    let props = ClientConnection::new(db.clone(), actors);
    let task = tokio::spawn(async move {
        let mut server_transport = server_transport;
        connect_client(props, &mut server_transport).await;
    });

    test_transport
        .send(auth::Inbound::AuthChallengeRequest {
            pubkey: new_key.verifying_key().into(),
            metadata: requested,
        })
        .await
        .unwrap();

    let response = test_transport
        .recv()
        .await
        .expect("should receive auth rejection");
    assert!(matches!(response, Err(auth::Error::IntegrityCheckFailed)));

    task.await.unwrap();
}
