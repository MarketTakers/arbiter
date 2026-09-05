use super::{EtherTransfer, Settings};
use crate::{
    db::{
        self, DatabaseConnection,
        models::{
            EvmBasicGrant, EvmWalletAccess, EvmWalletId, NewEvmBasicGrant, NewEvmTransactionLog,
            SqliteTimestamp,
        },
        schema::{evm_basic_grant, evm_transaction_log},
    },
    evm::{
        policies::{
            CombinedSettings, EvalContext, EvalViolation, Grant, Policy, SharedGrantSettings,
            VolumeRateLimit,
        },
        utils,
    },
};

use alloy::primitives::{Address, Bytes, U256, address};
use chrono::{Duration, Utc};
use diesel::{SelectableHelper, insert_into};
use diesel_async::RunQueryDsl;

const WALLET_ACCESS_ID: i32 = 1;
const CHAIN_ID: alloy::primitives::ChainId = 1;

const ALLOWED: Address = address!("1111111111111111111111111111111111111111");
const OTHER: Address = address!("2222222222222222222222222222222222222222");

fn ctx(to: Address, value: U256) -> EvalContext {
    EvalContext {
        target: EvmWalletAccess {
            id: WALLET_ACCESS_ID,
            wallet_id: EvmWalletId::from_raw(10),
            client_id: 20,
            created_at: SqliteTimestamp(Utc::now()),
        },
        chain: CHAIN_ID,
        to,
        value,
        calldata: Bytes::new(),
        max_fee_per_gas: 0,
        max_priority_fee_per_gas: 0,
    }
}

async fn insert_basic(conn: &mut DatabaseConnection, revoked: bool) -> EvmBasicGrant {
    insert_into(evm_basic_grant::table)
        .values(NewEvmBasicGrant {
            wallet_access_id: WALLET_ACCESS_ID,
            chain_id: CHAIN_ID.into(),
            valid_from: None,
            valid_until: None,
            max_gas_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            rate_limit_count: None,
            rate_limit_window_secs: None,
            revoked_at: revoked.then(|| SqliteTimestamp(Utc::now())),
        })
        .returning(EvmBasicGrant::as_select())
        .get_result(conn)
        .await
        .unwrap()
}

fn make_settings(targets: Vec<Address>, max_volume: u64) -> Settings {
    Settings {
        target: targets,
        limit: VolumeRateLimit {
            max_volume: U256::from(max_volume),
            window: Duration::hours(1),
        },
    }
}

fn shared() -> SharedGrantSettings {
    SharedGrantSettings {
        wallet_access_id: WALLET_ACCESS_ID,
        chain: CHAIN_ID,
        valid_from: None,
        valid_until: None,
        revoked_at: None,
        max_gas_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        rate_limit: None,
    }
}

#[test]
fn analyze_matches_empty_calldata() {
    let m = EtherTransfer::analyze(&ctx(ALLOWED, U256::from(1_000u64))).unwrap();
    assert_eq!(m.to, ALLOWED);
    assert_eq!(m.value, U256::from(1_000u64));
}

#[test]
fn analyze_rejects_nonempty_calldata() {
    let context = EvalContext {
        calldata: Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]),
        ..ctx(ALLOWED, U256::from(1u64))
    };
    assert!(EtherTransfer::analyze(&context).is_none());
}

#[tokio::test]
async fn evaluate_passes_for_allowed_target() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let grant = Grant {
        id: 999,
        common_settings_id: 999,
        settings: CombinedSettings {
            shared: shared(),
            specific: make_settings(vec![ALLOWED], 1_000_000),
        },
    };
    let context = ctx(ALLOWED, U256::from(100u64));
    let m = EtherTransfer::analyze(&context).unwrap();
    let v = EtherTransfer::evaluate(&context, &m, &grant, &mut *conn)
        .await
        .unwrap();
    assert!(v.is_empty());
}

#[tokio::test]
async fn evaluate_rejects_disallowed_target() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let grant = Grant {
        id: 999,
        common_settings_id: 999,
        settings: CombinedSettings {
            shared: shared(),
            specific: make_settings(vec![ALLOWED], 1_000_000),
        },
    };
    let context = ctx(OTHER, U256::from(100u64));
    let m = EtherTransfer::analyze(&context).unwrap();
    let v = EtherTransfer::evaluate(&context, &m, &grant, &mut *conn)
        .await
        .unwrap();
    assert!(
        v.iter()
            .any(|e| matches!(e, EvalViolation::InvalidTarget { .. }))
    );
}

#[tokio::test]
async fn evaluate_passes_when_volume_within_limit() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let basic = insert_basic(&mut conn, false).await;
    let settings = make_settings(vec![ALLOWED], 1_000);
    let grant_id = EtherTransfer::create_grant(&basic, &settings, &mut *conn)
        .await
        .unwrap();

    insert_into(evm_transaction_log::table)
        .values(NewEvmTransactionLog {
            grant_id,
            wallet_access_id: WALLET_ACCESS_ID,
            chain_id: CHAIN_ID.into(),
            eth_value: utils::u256_to_bytes(U256::from(500u64)).to_vec(),
            signed_at: SqliteTimestamp(Utc::now()),
        })
        .execute(&mut *conn)
        .await
        .unwrap();

    let grant = Grant {
        id: grant_id,
        common_settings_id: basic.id,
        settings: CombinedSettings {
            shared: shared(),
            specific: settings,
        },
    };
    let context = ctx(ALLOWED, U256::from(100u64));
    let m = EtherTransfer::analyze(&context).unwrap();
    let v = EtherTransfer::evaluate(&context, &m, &grant, &mut *conn)
        .await
        .unwrap();
    assert!(
        !v.iter()
            .any(|e| matches!(e, EvalViolation::VolumetricLimitExceeded))
    );
}

#[tokio::test]
async fn evaluate_rejects_volume_over_limit() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let basic = insert_basic(&mut conn, false).await;
    let settings = make_settings(vec![ALLOWED], 1_000);
    let grant_id = EtherTransfer::create_grant(&basic, &settings, &mut *conn)
        .await
        .unwrap();

    insert_into(evm_transaction_log::table)
        .values(NewEvmTransactionLog {
            grant_id,
            wallet_access_id: WALLET_ACCESS_ID,
            chain_id: CHAIN_ID.into(),
            eth_value: utils::u256_to_bytes(U256::from(1_000u64)).to_vec(),
            signed_at: SqliteTimestamp(Utc::now()),
        })
        .execute(&mut *conn)
        .await
        .unwrap();

    let grant = Grant {
        id: grant_id,
        common_settings_id: basic.id,
        settings: CombinedSettings {
            shared: shared(),
            specific: settings,
        },
    };
    let context = ctx(ALLOWED, U256::from(1u64));
    let m = EtherTransfer::analyze(&context).unwrap();
    let v = EtherTransfer::evaluate(&context, &m, &grant, &mut *conn)
        .await
        .unwrap();
    assert!(
        v.iter()
            .any(|e| matches!(e, EvalViolation::VolumetricLimitExceeded))
    );
}

#[tokio::test]
async fn evaluate_passes_at_exactly_volume_limit() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let basic = insert_basic(&mut conn, false).await;
    let settings = make_settings(vec![ALLOWED], 1_000);
    let grant_id = EtherTransfer::create_grant(&basic, &settings, &mut *conn)
        .await
        .unwrap();

    // Exactly at the limit including current transfer — check is `>`, so this should not violate
    insert_into(evm_transaction_log::table)
        .values(NewEvmTransactionLog {
            grant_id,
            wallet_access_id: WALLET_ACCESS_ID,
            chain_id: CHAIN_ID.into(),
            eth_value: utils::u256_to_bytes(U256::from(900u64)).to_vec(),
            signed_at: SqliteTimestamp(Utc::now()),
        })
        .execute(&mut *conn)
        .await
        .unwrap();

    let grant = Grant {
        id: grant_id,
        common_settings_id: basic.id,
        settings: CombinedSettings {
            shared: shared(),
            specific: settings,
        },
    };
    let context = ctx(ALLOWED, U256::from(100u64));
    let m = EtherTransfer::analyze(&context).unwrap();
    let v = EtherTransfer::evaluate(&context, &m, &grant, &mut *conn)
        .await
        .unwrap();
    assert!(
        !v.iter()
            .any(|e| matches!(e, EvalViolation::VolumetricLimitExceeded))
    );
}

#[tokio::test]
async fn try_find_grant_roundtrip() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let basic = insert_basic(&mut conn, false).await;
    let settings = make_settings(vec![ALLOWED], 1_000_000);
    EtherTransfer::create_grant(&basic, &settings, &mut *conn)
        .await
        .unwrap();

    let found = EtherTransfer::try_find_grant(&ctx(ALLOWED, U256::from(1u64)), &mut *conn)
        .await
        .unwrap();

    assert!(found.is_some());
    let g = found.unwrap();
    assert_eq!(g.settings.specific.target, vec![ALLOWED]);
    assert_eq!(
        g.settings.specific.limit.max_volume,
        U256::from(1_000_000u64)
    );
}

#[tokio::test]
async fn try_find_grant_revoked_returns_none() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let basic = insert_basic(&mut conn, true).await;
    let settings = make_settings(vec![ALLOWED], 1_000_000);
    EtherTransfer::create_grant(&basic, &settings, &mut *conn)
        .await
        .unwrap();

    let found = EtherTransfer::try_find_grant(&ctx(ALLOWED, U256::from(1u64)), &mut *conn)
        .await
        .unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn try_find_grant_wrong_target_returns_none() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let basic = insert_basic(&mut conn, false).await;
    let settings = make_settings(vec![ALLOWED], 1_000_000);
    EtherTransfer::create_grant(&basic, &settings, &mut *conn)
        .await
        .unwrap();

    let found = EtherTransfer::try_find_grant(&ctx(OTHER, U256::from(1u64)), &mut *conn)
        .await
        .unwrap();
    assert!(found.is_none());
}

proptest::proptest! {
    #[test]
    fn target_order_does_not_affect_hash(
        raw_addrs in proptest::collection::vec(proptest::prelude::any::<[u8; 20]>(), 0..8),
        seed in proptest::prelude::any::<u64>(),
        max_volume in proptest::prelude::any::<u64>(),
        window_secs in 1i64..=86400,
    ) {
        use rand::{SeedableRng, seq::SliceRandom};
        use sha2::Digest;
        use arbiter_crypto::hashing::Hashable;

        let addrs: Vec<Address> = raw_addrs.iter().map(|b| Address::from(*b)).collect();
        let mut shuffled = addrs.clone();
        shuffled.shuffle(&mut rand::rngs::StdRng::seed_from_u64(seed));

        let limit = VolumeRateLimit {
            max_volume: U256::from(max_volume),
            window: Duration::seconds(window_secs),
        };

        let mut h1 = sha2::Sha256::new();
        Settings { target: addrs, limit: limit.clone() }.hash(&mut h1);

        let mut h2 = sha2::Sha256::new();
        Settings { target: shuffled, limit }.hash(&mut h2);

        proptest::prop_assert_eq!(h1.finalize(), h2.finalize());
    }
}

#[tokio::test]
async fn find_all_grants_empty_db() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();
    let all = EtherTransfer::find_all_grants(&mut *conn).await.unwrap();
    assert!(all.is_empty());
}

#[tokio::test]
async fn find_all_grants_excludes_revoked() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let settings = make_settings(vec![ALLOWED], 1_000_000);
    let active = insert_basic(&mut conn, false).await;
    EtherTransfer::create_grant(&active, &settings, &mut *conn)
        .await
        .unwrap();
    let revoked = insert_basic(&mut conn, true).await;
    EtherTransfer::create_grant(&revoked, &settings, &mut *conn)
        .await
        .unwrap();

    let all = EtherTransfer::find_all_grants(&mut *conn).await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].settings.specific.target, vec![ALLOWED]);
}

#[tokio::test]
async fn find_all_grants_multiple_targets() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let basic = insert_basic(&mut conn, false).await;
    let settings = make_settings(vec![ALLOWED, OTHER], 1_000_000);
    EtherTransfer::create_grant(&basic, &settings, &mut *conn)
        .await
        .unwrap();

    let all = EtherTransfer::find_all_grants(&mut *conn).await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].settings.specific.target.len(), 2);
    assert_eq!(
        all[0].settings.specific.limit.max_volume,
        U256::from(1_000_000u64)
    );
}

#[tokio::test]
async fn find_all_grants_multiple_grants() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let basic1 = insert_basic(&mut conn, false).await;
    EtherTransfer::create_grant(&basic1, &make_settings(vec![ALLOWED], 500), &mut *conn)
        .await
        .unwrap();
    let basic2 = insert_basic(&mut conn, false).await;
    EtherTransfer::create_grant(&basic2, &make_settings(vec![OTHER], 1_000), &mut *conn)
        .await
        .unwrap();

    let all = EtherTransfer::find_all_grants(&mut *conn).await.unwrap();
    assert_eq!(all.len(), 2);
}
