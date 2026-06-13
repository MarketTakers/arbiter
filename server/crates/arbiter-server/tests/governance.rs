use arbiter_crypto::authn::{self, GOVERNANCE_CONTEXT};
use arbiter_server::{
    actors::{
        GlobalActors,
        proposal_manager::{
            CancelRecoveryWakeup, CastRecoveryVote, CastVote, CreateProposal,
            Error as ProposalError, ExpireStale, ProposalKind, QueryPending,
            RequestRecoveryWakeup, VoteOutcome,
        },
    },
    crypto::KeyCell,
    db,
};
use arbiter_server::actors::vault::Bootstrap;
use arbiter_server::db::schema::{
    aead_encrypted, evm_basic_grant, evm_wallet, evm_wallet_access, operator_identity,
    proposal_result, recovery_operator_identity,
};
use diesel::{ExpressionMethods, QueryDsl, insert_into};
use diesel_async::RunQueryDsl;

async fn register_operator(db: &db::DatabasePool, pubkey: &authn::PublicKey) -> i32 {
    let mut conn = db.get().await.unwrap();
    insert_into(operator_identity::table)
        .values(operator_identity::public_key.eq(pubkey.to_bytes()))
        .returning(operator_identity::id)
        .get_result::<i32>(&mut conn)
        .await
        .unwrap()
}

async fn register_recovery_operator(db: &db::DatabasePool, pubkey: &authn::PublicKey) -> i32 {
    let mut conn = db.get().await.unwrap();
    insert_into(recovery_operator_identity::table)
        .values(recovery_operator_identity::public_key.eq(pubkey.to_bytes()))
        .returning(recovery_operator_identity::id)
        .get_result::<i32>(&mut conn)
        .await
        .unwrap()
}

/// Backdates a wakeup request so it appears to have passed the 14-day window.
async fn insert_active_wakeup(db: &db::DatabasePool, operator_id: i32) {
    let mut conn = db.get().await.unwrap();
    diesel::sql_query(format!(
        "INSERT INTO recovery_wakeup_request (requested_by, requested_at) \
         VALUES ({operator_id}, unixepoch('now') - 14*24*3600 - 1)"
    ))
    .execute(&mut conn)
    .await
    .unwrap();
}

fn make_vote_message(proposal_id: i32, approve: bool) -> Vec<u8> {
    let mut msg = Vec::with_capacity(9);
    msg.extend_from_slice(&(proposal_id as i64).to_be_bytes());
    msg.push(u8::from(approve));
    msg
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
            kind: ProposalKind::ApproveSdkClient { client_id: 42 },
            initiator_id: 1,
            ttl_secs: None,
        })
        .await
        .unwrap();

    assert!(proposal_id > 0);
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
            kind: ProposalKind::ApproveSdkClient { client_id },
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();

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

    assert_eq!(outcome, VoteOutcome::QuorumApproved);
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
            kind: ProposalKind::ApproveSdkClient { client_id },
            initiator_id: op1,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = key1.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();

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
            kind: ProposalKind::ApproveSdkClient { client_id },
            initiator_id: op,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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
    let sig2 = key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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
            kind: ProposalKind::ApproveSdkClient { client_id },
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
            kind: ProposalKind::ApproveSdkClient { client_id: client_id1 },
            initiator_id: op,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let p2 = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveSdkClient { client_id: client_id2 },
            initiator_id: op,
            ttl_secs: None,
        })
        .await
        .unwrap();

    // Vote on p1 — with 1 operator this reaches quorum (QuorumApproved)
    let msg = make_vote_message(p1, true);
    let sig = signing_key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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
    assert_eq!(outcome, VoteOutcome::QuorumApproved);

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
async fn expire_stale_marks_old_proposals_expired() {
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

    // Create proposal with ttl_secs = -1 so it's immediately expired
    let _proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveSdkClient { client_id },
            initiator_id: op,
            ttl_secs: Some(-1),
        })
        .await
        .unwrap();

    let expired = actors
        .proposal_manager
        .ask(ExpireStale)
        .await
        .unwrap();
    assert_eq!(expired, 1);

    let pending = actors
        .proposal_manager
        .ask(QueryPending { operator_id: op })
        .await
        .unwrap();
    assert!(pending.is_empty());
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
            kind: ProposalKind::ApproveSdkClient { client_id },
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = op_key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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

    assert_eq!(outcome, VoteOutcome::QuorumApproved);

    let mut conn = db.get().await.unwrap();
    let count: i64 = integrity_envelope::table
        .filter(integrity_envelope::entity_kind.eq("client_credentials"))
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(count, 1);
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
            kind: ProposalKind::GrantWalletAccess { wallet_id, client_id },
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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

    assert_eq!(outcome, VoteOutcome::QuorumApproved);

    let mut conn = db.get().await.unwrap();
    let count: i64 = evm_wallet_access::table
        .filter(evm_wallet_access::wallet_id.eq(wallet_id))
        .filter(evm_wallet_access::client_id.eq(client_id))
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn approve_persistent_grant_creates_basic_grant_row() {
    use arbiter_proto::proto::operator::governance::{
        ApprovePersistentGrantPayload, EtherTransferSpecProto, VolumeLimitProto,
        approve_persistent_grant_payload::Specific,
    };
    use prost::Message as _;

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

    let payload = ApprovePersistentGrantPayload {
        wallet_access_id,
        chain_id: 1,
        valid_from_secs: None,
        valid_until_secs: None,
        max_gas_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        rate_limit: None,
        specific: Some(Specific::EtherTransfer(EtherTransferSpecProto {
            targets: vec![vec![0u8; 20]],
            limit: Some(VolumeLimitProto {
                max_volume: alloy::primitives::U256::from(1_000_000u64).to_be_bytes_vec(),
                window_secs: 86400,
            }),
        })),
    };

    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApprovePersistentGrant { payload_bytes: payload.encode_to_vec() },
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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

    assert_eq!(outcome, VoteOutcome::QuorumApproved);

    let mut conn = db.get().await.unwrap();
    let count: i64 = evm_basic_grant::table
        .filter(evm_basic_grant::wallet_access_id.eq(wallet_access_id))
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn approve_one_off_transaction_stores_result() {
    use arbiter_proto::proto::operator::governance::ApproveOneOffTransactionPayload;
    use arbiter_server::actors::evm::{Generate, OperatorCreateGrant};
    use arbiter_server::evm::policies::{
        SharedGrantSettings, SpecificGrant, VolumeRateLimit, ether_transfer,
    };
    use alloy::primitives::{Address, U256};
    use chrono::Duration;
    use prost::Message as _;

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

    // Encode the one-off transaction payload
    let payload = ApproveOneOffTransactionPayload {
        client_id,
        wallet_address: wallet_address.as_slice().to_vec(),
        chain_id: 1,
        nonce: 0,
        gas_limit: 21000,
        max_fee_per_gas: 1u128.to_be_bytes().to_vec(),
        max_priority_fee_per_gas: 1u128.to_be_bytes().to_vec(),
        to: to_address.as_slice().to_vec(),
        value: U256::from(1u64).to_be_bytes_vec(),
        input: vec![],
    };

    let proposal_id = actors
        .proposal_manager
        .ask(CreateProposal {
            kind: ProposalKind::ApproveOneOffTransaction { payload_bytes: payload.encode_to_vec() },
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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

    assert_eq!(outcome, VoteOutcome::QuorumApproved);

    let mut conn = db.get().await.unwrap();
    let count: i64 = proposal_result::table
        .filter(proposal_result::proposal_id.eq(proposal_id))
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn replace_operator_inserts_identity_row() {
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
            kind: ProposalKind::ReplaceOperator { new_pubkey },
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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

    assert_eq!(outcome, VoteOutcome::QuorumApproved);

    let mut conn = db.get().await.unwrap();
    let count: i64 = operator_identity::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(count, 2); // original + new
}

#[tokio::test]
async fn update_shamir_parameters_reaches_quorum() {
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
            kind: ProposalKind::UpdateShamirParameters { new_n: 5 },
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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

    assert_eq!(outcome, VoteOutcome::QuorumApproved);
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
            kind: ProposalKind::ReplaceOperator { new_pubkey },
            initiator_id: op1,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let cast = |op_id, key: &authn::SigningKey| {
        let actors = actors.clone();
        let sig = key.sign_message(&make_vote_message(proposal_id, true), GOVERNANCE_CONTEXT).unwrap();
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
    assert_eq!(cast(op3, &key3).await, VoteOutcome::QuorumApproved);
}

#[tokio::test]
async fn approve_server_update_reaches_quorum() {
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
            kind: ProposalKind::ApproveServerUpdate,
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = signing_key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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

    assert_eq!(outcome, VoteOutcome::QuorumApproved);
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
            kind: ProposalKind::ReplaceOperator { new_pubkey },
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = rec_key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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
            kind: ProposalKind::ApproveSdkClient { client_id },
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    let msg = make_vote_message(proposal_id, true);
    let sig = rec_key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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
            kind: ProposalKind::ReplaceOperator { new_pubkey },
            initiator_id: op_id,
            ttl_secs: None,
        })
        .await
        .unwrap();

    // Ordinary operator approves — still pending (needs recovery too)
    let msg = make_vote_message(proposal_id, true);
    let sig = op_key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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
    let sig = rec_key.sign_message(&msg, GOVERNANCE_CONTEXT).unwrap();
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
    assert_eq!(outcome, VoteOutcome::QuorumApproved);
}
