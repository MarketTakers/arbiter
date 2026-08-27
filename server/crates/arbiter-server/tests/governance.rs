use arbiter_crypto::authn::{self, SigningContext};
// The tests must sign exactly what the server verifies, so they share the encoder.
use arbiter_server::crypto::governance::vote_message as make_vote_message;
use arbiter_server::{
    actors::{
        GlobalActors,
        proposal_manager::{
            CancelRecoveryWakeup, CastRecoveryVote, CastVote, CreateProposal,
            Error as ProposalError, MAX_TTL_SECS, QueryPending, RequestRecoveryWakeup, VoteOutcome,
        },
    },
    crypto::KeyCell,
    db::{
        self,
        models::{OperatorIdentityId, RecoveryOperatorIdentityId},
        proposal::{
            ProposalKind, approve_sdk_client, grant_wallet_access, one_off_transaction,
            persistent_grant, replace_operator,
        },
    },
};
use arbiter_server::actors::vault::Bootstrap;
use arbiter_server::db::schema::{
    aead_encrypted, evm_basic_grant, evm_wallet, evm_wallet_access, operator_identity,
    proposal_one_off_transaction_result, recovery_operator_identity,
};
use diesel::{ExpressionMethods, QueryDsl, insert_into};
use diesel_async::RunQueryDsl;

/// Retries `probe` until it yields a value, then returns it.
///
/// Outcome execution is asynchronous: `CastVote` answers as soon as the quorum is
/// recorded, and the actor that owns the kind runs afterwards off the message bus. Tests
/// therefore wait for the effect instead of reading the database straight after the vote.
async fn eventually<T, F, Fut>(what: &str, mut probe: F) -> T
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Option<T>>,
{
    for _ in 0..100 {
        if let Some(value) = probe().await {
            return value;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("{what} did not happen within 2s");
}

async fn register_operator(db: &db::DatabasePool, pubkey: &authn::PublicKey) -> OperatorIdentityId {
    let mut conn = db.get().await.unwrap();
    insert_into(operator_identity::table)
        .values(operator_identity::public_key.eq(pubkey.to_bytes()))
        .returning(operator_identity::id)
        .get_result::<OperatorIdentityId>(&mut conn)
        .await
        .unwrap()
}

async fn register_recovery_operator(
    db: &db::DatabasePool,
    pubkey: &authn::PublicKey,
) -> RecoveryOperatorIdentityId {
    let mut conn = db.get().await.unwrap();
    insert_into(recovery_operator_identity::table)
        .values(recovery_operator_identity::public_key.eq(pubkey.to_bytes()))
        .returning(recovery_operator_identity::id)
        .get_result::<RecoveryOperatorIdentityId>(&mut conn)
        .await
        .unwrap()
}

/// Backdates a wakeup request so it appears to have passed the 14-day window.
async fn insert_active_wakeup(db: &db::DatabasePool, operator_id: OperatorIdentityId) {
    let mut conn = db.get().await.unwrap();
    diesel::sql_query(format!(
        "INSERT INTO recovery_wakeup_request (requested_by, requested_at) \
         VALUES ({}, unixepoch('now') - 14*24*3600 - 1)",
        operator_id.to_raw()
    ))
    .execute(&mut conn)
    .await
    .unwrap();
}

async fn insert_evm_wallet(db: &db::DatabasePool) -> i32 {
    let mut conn = db.get().await.unwrap();
    let aead_id: i32 = insert_into(aead_encrypted::table)
        .values((
            aead_encrypted::current_nonce.eq(vec![0u8; 4]),
            aead_encrypted::ciphertext.eq(vec![0u8; 32]),
            aead_encrypted::tag.eq(vec![0u8; 16]),
            aead_encrypted::associated_root_key_id.eq(0i32),
        ))
        .returning(aead_encrypted::id)
        .get_result::<i32>(&mut conn)
        .await
        .unwrap();
    insert_into(evm_wallet::table)
        .values((
            evm_wallet::address.eq(vec![0u8; 20]),
            evm_wallet::aead_encrypted_id.eq(aead_id),
        ))
        .returning(evm_wallet::id)
        .get_result::<i32>(&mut conn)
        .await
        .unwrap()
}

async fn insert_unapproved_client(db: &db::DatabasePool, pubkey: &authn::PublicKey) -> i32 {
    use arbiter_server::db::schema::{client_metadata, program_client};
    let mut conn = db.get().await.unwrap();
    let metadata_id: i32 = insert_into(client_metadata::table)
        .values((
            client_metadata::name.eq("test-client"),
            client_metadata::description.eq(Option::<String>::None),
            client_metadata::version.eq(Option::<String>::None),
        ))
        .returning(client_metadata::id)
        .get_result(&mut conn)
        .await
        .unwrap();

    insert_into(program_client::table)
        .values((
            program_client::public_key.eq(pubkey.to_bytes()),
            program_client::metadata_id.eq(metadata_id),
        ))
        .returning(program_client::id)
        .get_result(&mut conn)
        .await
        .unwrap()
}

#[tokio::test]
async fn create_proposal_returns_id() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap {
            seal_key: KeyCell::from([0u8; 32]),
        })
        .await
        .unwrap();

    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveSdkClient(approve_sdk_client::Settings { client_id: 42 }),
            initiator_id: OperatorIdentityId::from_raw(1),
            ttl_secs: None,
        })
        .await
        .unwrap();

    assert!(proposal_id.to_raw() > 0);
}

#[tokio::test]
async fn create_proposal_caps_the_ttl() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap {
            seal_key: KeyCell::from([0u8; 32]),
        })
        .await
        .unwrap();

    let key = authn::SigningKey::generate();
    let op = register_operator(&db, &key.public_key()).await;

    let create = async |ttl: u32| {
        actors
            .proposal_manager
            .ask(CreateProposal {
                kind: ProposalKind::ApproveSdkClient(approve_sdk_client::Settings { client_id: 1 }),
                initiator_id: op,
                ttl_secs: Some(ttl),
            })
            .await
    };

    // The boundary itself must still be accepted: the check is `>`, not `>=`.
    create(MAX_TTL_SECS)
        .await
        .expect("a TTL at the ceiling must be accepted");

    assert!(matches!(
        create(MAX_TTL_SECS + 1).await,
        Err(kameo::error::SendError::HandlerError(
            ProposalError::TtlTooLong
        ))
    ));
}

#[tokio::test]
async fn single_operator_vote_reaches_quorum() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) })
        .await
        .unwrap();

    let signing_key = authn::SigningKey::generate();
    let op_id = register_operator(&db, &signing_key.public_key()).await;

    let client_key = authn::SigningKey::generate();
    let client_id = insert_unapproved_client(&db, &client_key.public_key()).await;

    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveSdkClient(approve_sdk_client::Settings { client_id }),
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();

    let outcome = actors
        .proposal_manager
        .ask(CastVote {
            proposal_id,
            operator_id: op_id,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap();

    assert_eq!(outcome, VoteOutcome::Approved);
}

#[tokio::test]
async fn two_operator_first_vote_is_pending() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) })
        .await
        .unwrap();

    let key1 = authn::SigningKey::generate();
    let key2 = authn::SigningKey::generate();
    let op1 = register_operator(&db, &key1.public_key()).await;
    let _op2 = register_operator(&db, &key2.public_key()).await;
    let client_key = authn::SigningKey::generate();
    let client_id = insert_unapproved_client(&db, &client_key.public_key()).await;

    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveSdkClient(approve_sdk_client::Settings { client_id }),
            initiator_id: op1,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = key1
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();

    let outcome = actors
        .proposal_manager
        .ask(CastVote {
            proposal_id,
            operator_id: op1,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap();

    assert_eq!(outcome, VoteOutcome::Pending);
}

#[tokio::test]
async fn duplicate_vote_rejected() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) })
        .await
        .unwrap();

    let key = authn::SigningKey::generate();
    let op = register_operator(&db, &key.public_key()).await;

    let client_key = authn::SigningKey::generate();
    let client_id = insert_unapproved_client(&db, &client_key.public_key()).await;

    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveSdkClient(approve_sdk_client::Settings { client_id }),
            initiator_id: op,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    actors
        .proposal_manager
        .ask(CastVote {
            proposal_id,
            operator_id: op,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap();

    // Second vote same operator
    let sig2 = key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    let result = actors
        .proposal_manager
        .ask(CastVote {
            proposal_id,
            operator_id: op,
            approve: true,
            signature: sig2.to_bytes(),
        })
        .await;

    assert!(matches!(
        result,
        Err(kameo::error::SendError::HandlerError(ProposalError::AlreadyVoted))
    ));
}

#[tokio::test]
async fn invalid_signature_rejected() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) })
        .await
        .unwrap();

    let key = authn::SigningKey::generate();
    let op = register_operator(&db, &key.public_key()).await;
    let client_key = authn::SigningKey::generate();
    let client_id = insert_unapproved_client(&db, &client_key.public_key()).await;

    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveSdkClient(approve_sdk_client::Settings { client_id }),
            initiator_id: op,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let result = actors
        .proposal_manager
        .ask(CastVote {
            proposal_id,
            operator_id: op,
            approve: true,
            signature: vec![0u8; 32], // garbage
        })
        .await;

    assert!(matches!(
        result,
        Err(kameo::error::SendError::HandlerError(ProposalError::InvalidSignature))
    ));
}

#[tokio::test]
async fn query_pending_reports_a_tally_per_proposal() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap {
            seal_key: KeyCell::from([0u8; 32]),
        })
        .await
        .unwrap();

    // Three operators, so one vote stays below the 2-of-3 threshold and every
    // proposal is still pending when it is queried.
    let approver = authn::SigningKey::generate();
    let rejecter = authn::SigningKey::generate();
    let watcher = authn::SigningKey::generate();
    let approver_id = register_operator(&db, &approver.public_key()).await;
    let rejecter_id = register_operator(&db, &rejecter.public_key()).await;
    let watcher_id = register_operator(&db, &watcher.public_key()).await;

    let cast = async |proposal_id, voter_id, key: &authn::SigningKey, approve| {
        let sig = key
            .sign_message(
                &make_vote_message(proposal_id, approve),
                SigningContext::GovernanceVote,
            )
            .unwrap();
        actors
            .proposal_manager
            .ask(CastVote {
                proposal_id,
                operator_id: voter_id,
                approve,
                signature: sig.to_bytes(),
            })
            .await
            .unwrap()
    };

    let mut ids = Vec::new();
    for client_id in 1..=3 {
        let id = actors
            .proposal_manager
            .ask(CreateProposal {
                kind: ProposalKind::ApproveSdkClient(approve_sdk_client::Settings { client_id }),
                initiator_id: watcher_id,
                ttl_secs: None,
            })
            .await
            .unwrap();
        ids.push(id);
    }

    // First proposal: one approval. Second: one rejection. Third: neither.
    assert_eq!(
        cast(ids[0], approver_id, &approver, true).await,
        VoteOutcome::Pending
    );
    assert_eq!(
        cast(ids[1], rejecter_id, &rejecter, false).await,
        VoteOutcome::Pending
    );

    let summaries = actors
        .proposal_manager
        .ask(QueryPending {
            operator_id: watcher_id,
        })
        .await
        .unwrap();
    assert_eq!(summaries.len(), 3, "the watcher has voted on nothing");

    for summary in summaries {
        let (approve, reject) = match summary.id {
            id if id == ids[0] => (1, 0),
            id if id == ids[1] => (0, 1),
            _ => (0, 0),
        };
        assert_eq!(
            summary.approve_count, approve,
            "approvals of {:?}",
            summary.id
        );
        assert_eq!(
            summary.reject_count, reject,
            "rejections of {:?}",
            summary.id
        );
    }
}

#[tokio::test]
async fn query_pending_excludes_already_voted() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) })
        .await
        .unwrap();

    let signing_key = authn::SigningKey::generate();
    let op = register_operator(&db, &signing_key.public_key()).await;

    let client_key1 = authn::SigningKey::generate();
    let client_id1 = insert_unapproved_client(&db, &client_key1.public_key()).await;
    let client_key2 = authn::SigningKey::generate();
    let client_id2 = insert_unapproved_client(&db, &client_key2.public_key()).await;

    let p1 = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveSdkClient(approve_sdk_client::Settings { client_id: client_id1 }),
            initiator_id: op,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let p2 = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveSdkClient(approve_sdk_client::Settings { client_id: client_id2 }),
            initiator_id: op,
            ttl_secs: None,
        })
        .await
        .unwrap();

    // Vote on p1 — with 1 operator this reaches quorum (Approved)
    let msg = make_vote_message(p1, true);
    let sig = signing_key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    let outcome = actors
        .proposal_manager
        .ask(CastVote {
            proposal_id: p1,
            operator_id: op,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap();
    assert_eq!(outcome, VoteOutcome::Approved);

    // QueryPending should return only p2
    let pending = actors
        .proposal_manager
        .ask(QueryPending { operator_id: op })
        .await
        .unwrap();

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, p2);
}

#[tokio::test]
async fn expired_proposal_is_hidden_and_unvotable() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) })
        .await
        .unwrap();

    let signing_key = authn::SigningKey::generate();
    let op = register_operator(&db, &signing_key.public_key()).await;

    let client_key = authn::SigningKey::generate();
    let client_id = insert_unapproved_client(&db, &client_key.public_key()).await;

    // Create proposal with ttl_secs = 0 so it's immediately expired
    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveSdkClient(approve_sdk_client::Settings { client_id }),
            initiator_id: op,
            ttl_secs: Some(0),
        })
        .await
        .unwrap();

    // The row keeps status 'pending' (nothing sweeps it), but reads must skip it.
    let pending = actors
        .proposal_manager
        .ask(QueryPending { operator_id: op })
        .await
        .unwrap();
    assert!(pending.is_empty());

    // And the write path must refuse it rather than rely on a status flip.
    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    let result = actors
        .proposal_manager
        .ask(CastVote {
            proposal_id,
            operator_id: op,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await;

    assert!(matches!(
        result,
        Err(kameo::error::SendError::HandlerError(ProposalError::ProposalExpired))
    ));
}

#[tokio::test]
async fn approve_sdk_client_writes_integrity_envelope() {
    use arbiter_server::db::schema::integrity_envelope;

    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) })
        .await
        .unwrap();

    let client_key = authn::SigningKey::generate();
    let client_id = insert_unapproved_client(&db, &client_key.public_key()).await;

    let op_key = authn::SigningKey::generate();
    let op_id = register_operator(&db, &op_key.public_key()).await;

    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveSdkClient(approve_sdk_client::Settings { client_id }),
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = op_key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    let outcome = actors
        .proposal_manager
        .ask(CastVote {
            proposal_id,
            operator_id: op_id,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap();

    assert_eq!(outcome, VoteOutcome::Approved);

    eventually("the client's integrity envelope", || {
        let db = db.clone();
        async move {
            let mut conn = db.get().await.unwrap();
            let count: i64 = integrity_envelope::table
                .filter(integrity_envelope::entity_kind.eq("client_credentials"))
                .count()
                .get_result(&mut conn)
                .await
                .unwrap();
            (count == 1).then_some(())
        }
    })
    .await;
}

#[tokio::test]
async fn grant_wallet_access_on_quorum_approval() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) })
        .await
        .unwrap();

    let signing_key = authn::SigningKey::generate();
    let op_id = register_operator(&db, &signing_key.public_key()).await;

    let wallet_id = insert_evm_wallet(&db).await;
    let client_key = authn::SigningKey::generate();
    let client_id = insert_unapproved_client(&db, &client_key.public_key()).await;

    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::GrantWalletAccess(grant_wallet_access::Settings { wallet_id, client_id }),
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    let outcome = actors
        .proposal_manager
        .ask(CastVote {
            proposal_id,
            operator_id: op_id,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap();

    assert_eq!(outcome, VoteOutcome::Approved);

    eventually("the wallet access row", || {
        let db = db.clone();
        async move {
            let mut conn = db.get().await.unwrap();
            let count: i64 = evm_wallet_access::table
                .filter(evm_wallet_access::wallet_id.eq(wallet_id))
                .filter(evm_wallet_access::client_id.eq(client_id))
                .count()
                .get_result(&mut conn)
                .await
                .unwrap();
            (count == 1).then_some(())
        }
    })
    .await;
}

#[tokio::test]
async fn approve_persistent_grant_creates_basic_grant_row() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) })
        .await
        .unwrap();

    let signing_key = authn::SigningKey::generate();
    let op_id = register_operator(&db, &signing_key.public_key()).await;

    // Insert a dummy wallet and client, then a wallet_access row
    let wallet_id = insert_evm_wallet(&db).await;
    let client_key = authn::SigningKey::generate();
    let client_id = insert_unapproved_client(&db, &client_key.public_key()).await;

    let mut conn = db.get().await.unwrap();
    let wallet_access_id: i32 = insert_into(evm_wallet_access::table)
        .values((
            evm_wallet_access::wallet_id.eq(wallet_id),
            evm_wallet_access::client_id.eq(client_id),
        ))
        .returning(evm_wallet_access::id)
        .get_result(&mut conn)
        .await
        .unwrap();
    drop(conn);

    let grant = persistent_grant::Settings {
        wallet_access_id,
        chain_id: 1,
        valid_from_secs: None,
        valid_until_secs: None,
        max_gas_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        rate_limit: None,
        specific: persistent_grant::Specific::EtherTransfer {
            targets: vec![[0u8; 20]],
            limit: persistent_grant::VolumeLimit {
                max_volume: alloy::primitives::U256::from(1_000_000u64).to_be_bytes(),
                window_secs: 86400,
            },
        },
    };

    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApprovePersistentGrant(Box::new(grant)),
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    let outcome = actors
        .proposal_manager
        .ask(CastVote {
            proposal_id,
            operator_id: op_id,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap();

    assert_eq!(outcome, VoteOutcome::Approved);

    eventually("the basic grant row", || {
        let db = db.clone();
        async move {
            let mut conn = db.get().await.unwrap();
            let count: i64 = evm_basic_grant::table
                .filter(evm_basic_grant::wallet_access_id.eq(wallet_access_id))
                .count()
                .get_result(&mut conn)
                .await
                .unwrap();
            (count == 1).then_some(())
        }
    })
    .await;
}

#[tokio::test]
async fn approve_one_off_transaction_stores_result() {
    use alloy::primitives::{Address, U256};
    use arbiter_server::actors::evm::{Generate, OperatorCreateGrant};
    use arbiter_server::evm::policies::{
        SharedGrantSettings, SpecificGrant, VolumeRateLimit, ether_transfer,
    };
    use chrono::Duration;

    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) })
        .await
        .unwrap();

    let signing_key = authn::SigningKey::generate();
    let op_id = register_operator(&db, &signing_key.public_key()).await;

    // Create a real encrypted wallet
    let (wallet_id, wallet_address) = actors.evm.ask(Generate {}).await.unwrap();

    // Create a client and wallet_access
    let client_key = authn::SigningKey::generate();
    let client_id = insert_unapproved_client(&db, &client_key.public_key()).await;

    let mut conn = db.get().await.unwrap();
    let wallet_access_id: i32 = insert_into(evm_wallet_access::table)
        .values((
            evm_wallet_access::wallet_id.eq(wallet_id),
            evm_wallet_access::client_id.eq(client_id),
        ))
        .returning(evm_wallet_access::id)
        .get_result(&mut conn)
        .await
        .unwrap();
    drop(conn);

    // Create a grant that permits ether transfer to address zero
    let to_address = Address::ZERO;
    actors
        .evm
        .ask(OperatorCreateGrant {
            basic: SharedGrantSettings {
                wallet_access_id,
                chain: 1,
                valid_from: None,
                valid_until: None,
                max_gas_fee_per_gas: None,
                max_priority_fee_per_gas: None,
                rate_limit: None,
            },
            grant: SpecificGrant::EtherTransfer(ether_transfer::Settings {
                target: vec![to_address],
                limit: VolumeRateLimit {
                    max_volume: U256::from(1_000_000_000_000_000_000u128),
                    window: Duration::hours(24),
                },
            }),
        })
        .await
        .unwrap();

    let transaction = one_off_transaction::Settings {
        client_id,
        wallet_address: wallet_address.into(),
        chain_id: 1,
        nonce: 0,
        gas_limit: 21000,
        max_fee_per_gas: 1,
        max_priority_fee_per_gas: 1,
        to: to_address.into(),
        value: U256::from(1u64).to_be_bytes(),
        input: vec![],
    };

    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveOneOffTransaction(Box::new(transaction)),
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    let outcome = actors
        .proposal_manager
        .ask(CastVote {
            proposal_id,
            operator_id: op_id,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap();

    assert_eq!(outcome, VoteOutcome::Approved);

    let (r, s, y_parity): (Vec<u8>, Vec<u8>, i32) = eventually("the transaction signature", || {
        let db = db.clone();
        async move {
            let mut conn = db.get().await.unwrap();
            proposal_one_off_transaction_result::table
                .find(proposal_id)
                .select((
                    proposal_one_off_transaction_result::r,
                    proposal_one_off_transaction_result::s,
                    proposal_one_off_transaction_result::y_parity,
                ))
                .first(&mut conn)
                .await
                .ok()
        }
    })
    .await;

    assert_eq!(r.len(), 32, "r must be a 32-byte scalar");
    assert_eq!(s.len(), 32, "s must be a 32-byte scalar");
    assert!(y_parity == 0 || y_parity == 1, "y_parity must be a bit");
}

#[tokio::test]
async fn replace_operator_updates_pubkey_and_starts_rekey() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) })
        .await
        .unwrap();

    let signing_key = authn::SigningKey::generate();
    let op_id = register_operator(&db, &signing_key.public_key()).await;

    let new_op_key = authn::SigningKey::generate();
    let new_pubkey = new_op_key.public_key().to_bytes();

    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ReplaceOperator(replace_operator::Settings {
                old_operator_id: op_id,
                new_pubkey: new_pubkey.clone(),
            }),
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    let outcome = actors
        .proposal_manager
        .ask(CastVote {
            proposal_id,
            operator_id: op_id,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap();

    assert_eq!(outcome, VoteOutcome::Approved);

    eventually("the operator's public key to be replaced", || {
        let db = db.clone();
        let new_pubkey = new_pubkey.clone();
        async move {
            let mut conn = db.get().await.unwrap();
            let stored: Vec<u8> = operator_identity::table
                .filter(operator_identity::id.eq(op_id))
                .select(operator_identity::public_key)
                .first(&mut conn)
                .await
                .unwrap();
            (stored == new_pubkey).then_some(())
        }
    })
    .await;

    // The old identity row is updated in place, so no second operator appears.
    let mut conn = db.get().await.unwrap();
    let count: i64 = operator_identity::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn trigger_rekey_reaches_quorum() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) })
        .await
        .unwrap();

    let signing_key = authn::SigningKey::generate();
    let op_id = register_operator(&db, &signing_key.public_key()).await;

    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::TriggerRekey,
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    let outcome = actors
        .proposal_manager
        .ask(CastVote {
            proposal_id,
            operator_id: op_id,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap();

    assert_eq!(outcome, VoteOutcome::Approved);
}

#[tokio::test]
async fn key_rotation_requires_full_quorum() {
    // §3.3: ReplaceOperator needs all 3 operators to approve, not just shamir_threshold(3)=2
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors
        .vault
        .ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) })
        .await
        .unwrap();

    let key1 = authn::SigningKey::generate();
    let key2 = authn::SigningKey::generate();
    let key3 = authn::SigningKey::generate();
    let op1 = register_operator(&db, &key1.public_key()).await;
    let op2 = register_operator(&db, &key2.public_key()).await;
    let op3 = register_operator(&db, &key3.public_key()).await;

    let new_pubkey = authn::SigningKey::generate().public_key().to_bytes();
    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ReplaceOperator(replace_operator::Settings {
                old_operator_id: OperatorIdentityId::from_raw(1),
                new_pubkey,
            }),
            initiator_id: op1,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let cast = |op_id, key: &authn::SigningKey| {
        let actors = actors.clone();
        let sig = key
            .sign_message(&make_vote_message(proposal_id, true), SigningContext::GovernanceVote)
            .unwrap();
        async move {
            actors
                .proposal_manager
                .ask(CastVote { proposal_id, operator_id: op_id, approve: true, signature: sig.to_bytes() })
                .await
                .unwrap()
        }
    };

    // With shamir_threshold(3)=2, two approvals would suffice for a normal proposal.
    // For key rotation, they must not.
    assert_eq!(cast(op1, &key1).await, VoteOutcome::Pending);
    assert_eq!(cast(op2, &key2).await, VoteOutcome::Pending);
    assert_eq!(cast(op3, &key3).await, VoteOutcome::Approved);
}

// ─── §3.5 / §3.6 Recovery Operator tests ──────────────────────────────────

#[tokio::test]
async fn recovery_vote_rejected_when_sleeping() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors.vault.ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) }).await.unwrap();

    let op_key = authn::SigningKey::generate();
    let op_id = register_operator(&db, &op_key.public_key()).await;
    let rec_key = authn::SigningKey::generate();
    let rec_id = register_recovery_operator(&db, &rec_key.public_key()).await;

    let new_pubkey = authn::SigningKey::generate().public_key().to_bytes();
    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ReplaceOperator(replace_operator::Settings {
                old_operator_id: OperatorIdentityId::from_raw(1),
                new_pubkey,
            }),
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = rec_key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    let err = actors
        .proposal_manager
        .ask(CastRecoveryVote {
            proposal_id,
            recovery_operator_id: rec_id,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap_err();

    assert!(
        matches!(err, kameo::error::SendError::HandlerError(ProposalError::RecoveryNotActive)),
        "expected RecoveryNotActive, got {err:?}"
    );
}

#[tokio::test]
async fn recovery_vote_blocked_on_non_replace_proposal() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors.vault.ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) }).await.unwrap();

    let op_key = authn::SigningKey::generate();
    let op_id = register_operator(&db, &op_key.public_key()).await;
    let rec_key = authn::SigningKey::generate();
    let rec_id = register_recovery_operator(&db, &rec_key.public_key()).await;

    insert_active_wakeup(&db, op_id).await;

    let client_key = authn::SigningKey::generate();
    let client_id = insert_unapproved_client(&db, &client_key.public_key()).await;
    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveSdkClient(approve_sdk_client::Settings { client_id }),
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = rec_key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    let err = actors
        .proposal_manager
        .ask(CastRecoveryVote {
            proposal_id,
            recovery_operator_id: rec_id,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap_err();

    assert!(
        matches!(
            err,
            kameo::error::SendError::HandlerError(ProposalError::NotAllowedForRecoveryOperator)
        ),
        "expected NotAllowedForRecoveryOperator, got {err:?}"
    );
}

#[tokio::test]
async fn recovery_wakeup_can_be_cancelled() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors.vault.ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) }).await.unwrap();

    let key = authn::SigningKey::generate();
    let op_id = register_operator(&db, &key.public_key()).await;

    actors
        .proposal_manager
        .ask(RequestRecoveryWakeup { operator_id: op_id })
        .await
        .unwrap();

    actors
        .proposal_manager
        .ask(CancelRecoveryWakeup { operator_id: op_id })
        .await
        .unwrap();

    // Second request must succeed (previous one was cancelled)
    actors
        .proposal_manager
        .ask(RequestRecoveryWakeup { operator_id: op_id })
        .await
        .unwrap();
}

#[tokio::test]
async fn recovery_wakeup_prevents_duplicate_request() {
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors.vault.ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) }).await.unwrap();

    let key = authn::SigningKey::generate();
    let op_id = register_operator(&db, &key.public_key()).await;

    actors
        .proposal_manager
        .ask(RequestRecoveryWakeup { operator_id: op_id })
        .await
        .unwrap();

    let err = actors
        .proposal_manager
        .ask(RequestRecoveryWakeup { operator_id: op_id })
        .await
        .unwrap_err();

    assert!(
        matches!(err, kameo::error::SendError::HandlerError(ProposalError::WakeupAlreadyPending)),
        "expected WakeupAlreadyPending, got {err:?}"
    );
}

#[tokio::test]
async fn recovery_operator_vote_contributes_to_replace_quorum() {
    // 1 ordinary operator + 1 recovery operator; replace_operator needs both.
    let db = db::create_test_pool().await;
    let actors = GlobalActors::spawn(db.clone()).await.unwrap();
    actors.vault.ask(Bootstrap { seal_key: KeyCell::from([0u8; 32]) }).await.unwrap();

    let op_key = authn::SigningKey::generate();
    let op_id = register_operator(&db, &op_key.public_key()).await;
    let rec_key = authn::SigningKey::generate();
    let rec_id = register_recovery_operator(&db, &rec_key.public_key()).await;

    insert_active_wakeup(&db, op_id).await;

    let new_pubkey = authn::SigningKey::generate().public_key().to_bytes();
    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ReplaceOperator(replace_operator::Settings {
                old_operator_id: OperatorIdentityId::from_raw(1),
                new_pubkey,
            }),
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    // Ordinary operator approves — still pending (needs recovery too)
    let msg = make_vote_message(proposal_id, true);
    let sig = op_key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    let outcome = actors
        .proposal_manager
        .ask(CastVote {
            proposal_id,
            operator_id: op_id,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap();
    assert_eq!(outcome, VoteOutcome::Pending);

    // Recovery operator approves — now quorum is reached
    let sig = rec_key
        .sign_message(&msg, SigningContext::GovernanceVote)
        .unwrap();
    let outcome = actors
        .proposal_manager
        .ask(CastRecoveryVote {
            proposal_id,
            recovery_operator_id: rec_id,
            approve: true,
            signature: sig.to_bytes(),
        })
        .await
        .unwrap();
    assert_eq!(outcome, VoteOutcome::Approved);
}
