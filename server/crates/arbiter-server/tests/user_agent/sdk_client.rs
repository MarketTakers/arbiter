use arbiter_proto::proto::user_agent::{
    SdkClientApproveRequest, SdkClientError as ProtoSdkClientError, SdkClientRevokeRequest,
    UserAgentRequest, sdk_client_approve_response, sdk_client_list_response,
    sdk_client_revoke_response, user_agent_request::Payload as UserAgentRequestPayload,
    user_agent_response::Payload as UserAgentResponsePayload,
};
use arbiter_server::{
    actors::{
        GlobalActors,
        user_agent::{TransportResponseError, session::UserAgentSession},
    },
    db,
};

/// Shared helper: create a session and register a client pubkey via sdk_client_approve.
async fn make_session(db: &db::DatabasePool) -> UserAgentSession {
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    UserAgentSession::new_test(db.clone(), actors)
}

#[tokio::test]
#[test_log::test]
async fn test_sdk_client_approve_registers_client() {
    let db = db::create_test_pool().await;
    let mut session = make_session(&db).await;

    let pubkey = [0x42u8; 32];

    let response = session
        .process_transport_inbound(UserAgentRequest {
            payload: Some(UserAgentRequestPayload::SdkClientApprove(
                SdkClientApproveRequest {
                    pubkey: pubkey.to_vec(),
                },
            )),
        })
        .await
        .expect("handler should succeed");

    let entry = match response.payload.unwrap() {
        UserAgentResponsePayload::SdkClientApprove(resp) => match resp.result.unwrap() {
            sdk_client_approve_response::Result::Client(e) => e,
            sdk_client_approve_response::Result::Error(e) => {
                panic!("Expected Client, got error {:?}", e)
            }
        },
        other => panic!("Expected SdkClientApprove, got {other:?}"),
    };

    assert_eq!(entry.pubkey, pubkey.to_vec());
    assert!(entry.id > 0);
}

#[tokio::test]
#[test_log::test]
async fn test_sdk_client_approve_duplicate_returns_already_exists() {
    let db = db::create_test_pool().await;
    let mut session = make_session(&db).await;

    let pubkey = [0x11u8; 32];
    let req = UserAgentRequest {
        payload: Some(UserAgentRequestPayload::SdkClientApprove(
            SdkClientApproveRequest {
                pubkey: pubkey.to_vec(),
            },
        )),
    };

    session
        .process_transport_inbound(req.clone())
        .await
        .unwrap();

    let err = session
        .process_transport_inbound(req)
        .await
        .expect_err("second insert should return typed TransportResponseError");

    assert_eq!(
        err,
        TransportResponseError::SdkClientApprove(ProtoSdkClientError::AlreadyExists)
    );
}

#[tokio::test]
#[test_log::test]
async fn test_sdk_client_list_shows_registered_clients() {
    let db = db::create_test_pool().await;
    let mut session = make_session(&db).await;

    let pubkey_a = [0x0Au8; 32];
    let pubkey_b = [0x0Bu8; 32];

    for pubkey in [pubkey_a, pubkey_b] {
        session
            .process_transport_inbound(UserAgentRequest {
                payload: Some(UserAgentRequestPayload::SdkClientApprove(
                    SdkClientApproveRequest {
                        pubkey: pubkey.to_vec(),
                    },
                )),
            })
            .await
            .unwrap();
    }

    let response = session
        .process_transport_inbound(UserAgentRequest {
            payload: Some(UserAgentRequestPayload::SdkClientList(())),
        })
        .await
        .expect("list should succeed");

    let clients = match response.payload.unwrap() {
        UserAgentResponsePayload::SdkClientList(resp) => match resp.result.unwrap() {
            sdk_client_list_response::Result::Clients(list) => list.clients,
            sdk_client_list_response::Result::Error(e) => {
                panic!("Expected Clients, got error {:?}", e)
            }
        },
        other => panic!("Expected SdkClientList, got {other:?}"),
    };

    assert_eq!(clients.len(), 2);
    let pubkeys: Vec<Vec<u8>> = clients.into_iter().map(|e| e.pubkey).collect();
    assert!(pubkeys.contains(&pubkey_a.to_vec()));
    assert!(pubkeys.contains(&pubkey_b.to_vec()));
}

#[tokio::test]
#[test_log::test]
async fn test_sdk_client_revoke_removes_client() {
    let db = db::create_test_pool().await;
    let mut session = make_session(&db).await;

    let pubkey = [0xBBu8; 32];

    // Register a client and get its id
    let approve_response = session
        .process_transport_inbound(UserAgentRequest {
            payload: Some(UserAgentRequestPayload::SdkClientApprove(
                SdkClientApproveRequest {
                    pubkey: pubkey.to_vec(),
                },
            )),
        })
        .await
        .unwrap();

    let client_id = match approve_response.payload.unwrap() {
        UserAgentResponsePayload::SdkClientApprove(resp) => match resp.result.unwrap() {
            sdk_client_approve_response::Result::Client(e) => e.id,
            sdk_client_approve_response::Result::Error(e) => panic!("approve failed: {:?}", e),
        },
        other => panic!("{other:?}"),
    };

    // Revoke the client
    let revoke_response = session
        .process_transport_inbound(UserAgentRequest {
            payload: Some(UserAgentRequestPayload::SdkClientRevoke(
                SdkClientRevokeRequest { client_id },
            )),
        })
        .await
        .expect("revoke should succeed");

    match revoke_response.payload.unwrap() {
        UserAgentResponsePayload::SdkClientRevoke(resp) => match resp.result.unwrap() {
            sdk_client_revoke_response::Result::Ok(_) => {}
            sdk_client_revoke_response::Result::Error(e) => {
                panic!("Expected Ok, got error {:?}", e)
            }
        },
        other => panic!("Expected SdkClientRevoke, got {other:?}"),
    }

    // List should now be empty
    let list_response = session
        .process_transport_inbound(UserAgentRequest {
            payload: Some(UserAgentRequestPayload::SdkClientList(())),
        })
        .await
        .unwrap();

    let clients = match list_response.payload.unwrap() {
        UserAgentResponsePayload::SdkClientList(resp) => match resp.result.unwrap() {
            sdk_client_list_response::Result::Clients(list) => list.clients,
            sdk_client_list_response::Result::Error(e) => panic!("list error: {:?}", e),
        },
        other => panic!("{other:?}"),
    };
    assert!(clients.is_empty(), "client should be removed after revoke");
}

#[tokio::test]
#[test_log::test]
async fn test_sdk_client_revoke_not_found_returns_error() {
    let db = db::create_test_pool().await;
    let mut session = make_session(&db).await;

    let err = session
        .process_transport_inbound(UserAgentRequest {
            payload: Some(UserAgentRequestPayload::SdkClientRevoke(
                SdkClientRevokeRequest { client_id: 9999 },
            )),
        })
        .await
        .expect_err("missing client should return typed TransportResponseError");

    assert_eq!(
        err,
        TransportResponseError::SdkClientRevoke(ProtoSdkClientError::NotFound)
    );
}

#[tokio::test]
#[test_log::test]
async fn test_sdk_client_approve_rejected_client_cannot_auth() {
    // Verify the core flow: only pre-approved clients can authenticate
    use arbiter_proto::proto::client::{
        AuthChallengeRequest, ClientRequest, client_request::Payload as ClientRequestPayload,
        client_response::Payload as ClientResponsePayload,
    };
    use arbiter_proto::transport::Bi as _;
    use arbiter_server::actors::client::{ClientConnection, connect_client};

    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();

    let new_key = ed25519_dalek::SigningKey::generate(&mut rand::rng());
    let pubkey_bytes = new_key.verifying_key().to_bytes().to_vec();

    let (server_transport, mut test_transport) = super::common::ChannelTransport::<_, _>::new();
    let props = ClientConnection::new(db.clone(), Box::new(server_transport), actors.clone());
    let task = tokio::spawn(connect_client(props));

    test_transport
        .send(ClientRequest {
            payload: Some(ClientRequestPayload::AuthChallengeRequest(
                AuthChallengeRequest {
                    pubkey: pubkey_bytes.clone(),
                },
            )),
        })
        .await
        .unwrap();

    let response = test_transport.recv().await.unwrap().unwrap();
    assert!(
        matches!(
            response.payload.unwrap(),
            ClientResponsePayload::ClientConnectError(_)
        ),
        "unregistered client should be rejected"
    );

    task.await.unwrap();
}
