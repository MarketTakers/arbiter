use diesel::{ExpressionMethods as _, QueryDsl as _, dsl::insert_into};
use diesel_async::RunQueryDsl;
use kameo::actor::Spawn as _;

use crate::{
    actors::{evm::EvmActor, keyholder::KeyHolder},
    db::{self, models, schema},
};

#[tokio::test]
async fn delete_ether_grant_cleans_related_tables() {
    let db = db::create_test_pool().await;
    let keyholder = KeyHolder::spawn(KeyHolder::new(db.clone()).await.unwrap());
    let mut actor = EvmActor::new(keyholder, db.clone());

    let mut conn = db.get().await.unwrap();

    let basic_id: i32 = insert_into(schema::evm_basic_grant::table)
        .values(&models::NewEvmBasicGrant {
            wallet_access_id: 1,
            chain_id: 1,
            valid_from: None,
            valid_until: None,
            max_gas_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            rate_limit_count: None,
            rate_limit_window_secs: None,
            revoked_at: None,
        })
        .returning(schema::evm_basic_grant::id)
        .get_result(&mut conn)
        .await
        .unwrap();

    let limit_id: i32 = insert_into(schema::evm_ether_transfer_limit::table)
        .values(&models::NewEvmEtherTransferLimit {
            window_secs: 60,
            max_volume: vec![1],
        })
        .returning(schema::evm_ether_transfer_limit::id)
        .get_result(&mut conn)
        .await
        .unwrap();

    let ether_grant_id: i32 = insert_into(schema::evm_ether_transfer_grant::table)
        .values(&models::NewEvmEtherTransferGrant {
            basic_grant_id: basic_id,
            limit_id,
        })
        .returning(schema::evm_ether_transfer_grant::id)
        .get_result(&mut conn)
        .await
        .unwrap();

    insert_into(schema::evm_ether_transfer_grant_target::table)
        .values(&models::NewEvmEtherTransferGrantTarget {
            grant_id: ether_grant_id,
            address: vec![0u8; 20],
        })
        .execute(&mut conn)
        .await
        .unwrap();

    insert_into(schema::evm_transaction_log::table)
        .values(&models::NewEvmTransactionLog {
            grant_id: basic_id,
            wallet_access_id: 1,
            chain_id: 1,
            eth_value: vec![0],
            signed_at: models::SqliteTimestamp::now(),
        })
        .execute(&mut conn)
        .await
        .unwrap();

    insert_into(schema::integrity_envelope::table)
        .values(&models::NewIntegrityEnvelope {
            entity_kind: "EtherTransfer".to_owned(),
            entity_id: basic_id.to_be_bytes().to_vec(),
            payload_version: 1,
            key_version: 1,
            mac: vec![0u8; 32],
        })
        .execute(&mut conn)
        .await
        .unwrap();

    drop(conn);

    actor.useragent_delete_grant(basic_id).await.unwrap();

    // Idempotency: second delete should be a no-op.
    actor.useragent_delete_grant(basic_id).await.unwrap();

    let mut conn = db.get().await.unwrap();

    let basic_count: i64 = schema::evm_basic_grant::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(basic_count, 0);

    let ether_grant_count: i64 = schema::evm_ether_transfer_grant::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(ether_grant_count, 0);

    let target_count: i64 = schema::evm_ether_transfer_grant_target::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(target_count, 0);

    let limit_count: i64 = schema::evm_ether_transfer_limit::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(limit_count, 0);

    let log_count: i64 = schema::evm_transaction_log::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(log_count, 0);

    let envelope_count: i64 = schema::integrity_envelope::table
        .filter(schema::integrity_envelope::entity_kind.eq("EtherTransfer"))
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(envelope_count, 0);
}

#[tokio::test]
async fn delete_token_grant_cleans_related_tables() {
    let db = db::create_test_pool().await;
    let keyholder = KeyHolder::spawn(KeyHolder::new(db.clone()).await.unwrap());
    let mut actor = EvmActor::new(keyholder, db.clone());

    let mut conn = db.get().await.unwrap();

    let basic_id: i32 = insert_into(schema::evm_basic_grant::table)
        .values(&models::NewEvmBasicGrant {
            wallet_access_id: 1,
            chain_id: 1,
            valid_from: None,
            valid_until: None,
            max_gas_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            rate_limit_count: None,
            rate_limit_window_secs: None,
            revoked_at: None,
        })
        .returning(schema::evm_basic_grant::id)
        .get_result(&mut conn)
        .await
        .unwrap();

    let token_grant_id: i32 = insert_into(schema::evm_token_transfer_grant::table)
        .values(&models::NewEvmTokenTransferGrant {
            basic_grant_id: basic_id,
            token_contract: vec![1u8; 20],
            receiver: None,
        })
        .returning(schema::evm_token_transfer_grant::id)
        .get_result(&mut conn)
        .await
        .unwrap();

    insert_into(schema::evm_token_transfer_volume_limit::table)
        .values(&models::NewEvmTokenTransferVolumeLimit {
            grant_id: token_grant_id,
            window_secs: 60,
            max_volume: vec![1],
        })
        .execute(&mut conn)
        .await
        .unwrap();

    insert_into(schema::evm_token_transfer_volume_limit::table)
        .values(&models::NewEvmTokenTransferVolumeLimit {
            grant_id: token_grant_id,
            window_secs: 3600,
            max_volume: vec![2],
        })
        .execute(&mut conn)
        .await
        .unwrap();

    let tx_log_id: i32 = insert_into(schema::evm_transaction_log::table)
        .values(&models::NewEvmTransactionLog {
            grant_id: basic_id,
            wallet_access_id: 1,
            chain_id: 1,
            eth_value: vec![0],
            signed_at: models::SqliteTimestamp::now(),
        })
        .returning(schema::evm_transaction_log::id)
        .get_result(&mut conn)
        .await
        .unwrap();

    insert_into(schema::evm_token_transfer_log::table)
        .values(&models::NewEvmTokenTransferLog {
            grant_id: token_grant_id,
            log_id: tx_log_id,
            chain_id: 1,
            token_contract: vec![1u8; 20],
            recipient_address: vec![2u8; 20],
            value: vec![3],
        })
        .execute(&mut conn)
        .await
        .unwrap();

    insert_into(schema::integrity_envelope::table)
        .values(&models::NewIntegrityEnvelope {
            entity_kind: "TokenTransfer".to_owned(),
            entity_id: basic_id.to_be_bytes().to_vec(),
            payload_version: 1,
            key_version: 1,
            mac: vec![0u8; 32],
        })
        .execute(&mut conn)
        .await
        .unwrap();

    drop(conn);

    actor.useragent_delete_grant(basic_id).await.unwrap();

    let mut conn = db.get().await.unwrap();

    let basic_count: i64 = schema::evm_basic_grant::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(basic_count, 0);

    let token_grant_count: i64 = schema::evm_token_transfer_grant::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(token_grant_count, 0);

    let token_limits_count: i64 = schema::evm_token_transfer_volume_limit::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(token_limits_count, 0);

    let token_logs_count: i64 = schema::evm_token_transfer_log::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(token_logs_count, 0);

    let tx_logs_count: i64 = schema::evm_transaction_log::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(tx_logs_count, 0);

    let envelope_count: i64 = schema::integrity_envelope::table
        .filter(schema::integrity_envelope::entity_kind.eq("TokenTransfer"))
        .count()
        .get_result(&mut conn)
        .await
        .unwrap();
    assert_eq!(envelope_count, 0);
}
