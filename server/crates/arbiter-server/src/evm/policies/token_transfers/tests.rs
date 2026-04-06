use alloy::primitives::{Address, Bytes, U256, address};
use alloy::sol_types::SolCall;
use chrono::{Duration, Utc};
use diesel::{SelectableHelper, insert_into};
use diesel_async::RunQueryDsl;

use crate::db::{
    self, DatabaseConnection,
    models::{EvmBasicGrant, EvmWalletAccess, NewEvmBasicGrant, SqliteTimestamp},
    schema::evm_basic_grant,
};
use crate::evm::{
    abi::IERC20::transferCall,
    policies::{
        CombinedSettings, EvalContext, EvalViolation, Grant, Policy, SharedGrantSettings,
        VolumeRateLimit,
    },
    utils,
};

use super::{Settings, TokenTransfer};

// DAI on Ethereum mainnet — present in the static token registry
const CHAIN_ID: u64 = 1;
const DAI: Address = address!("6B175474E89094C44Da98b954EedeAC495271d0F");

const WALLET_ACCESS_ID: i32 = 1;

const RECIPIENT: Address = address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
const OTHER: Address = address!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
const UNKNOWN_TOKEN: Address = address!("cccccccccccccccccccccccccccccccccccccccc");

/// Encode `transfer(to, value)` raw params (no 4-byte selector).
/// `abi_decode_raw_validate` expects exactly this format.
fn transfer_calldata(to: Address, value: U256) -> Bytes {
    let mut raw = Vec::new();
    transferCall { to, value }.abi_encode_raw(&mut raw);
    Bytes::from(raw)
}

fn ctx(to: Address, calldata: Bytes) -> EvalContext {
    EvalContext {
        target: EvmWalletAccess {
            id: WALLET_ACCESS_ID,
            wallet_id: 10,
            client_id: 20,
            created_at: SqliteTimestamp(Utc::now()),
        },
        chain: CHAIN_ID,
        to,
        value: U256::ZERO,
        calldata,
        max_fee_per_gas: 0,
        max_priority_fee_per_gas: 0,
    }
}

async fn insert_basic(conn: &mut DatabaseConnection, revoked: bool) -> EvmBasicGrant {
    insert_into(evm_basic_grant::table)
        .values(NewEvmBasicGrant {
            wallet_access_id: WALLET_ACCESS_ID,
            chain_id: CHAIN_ID as i32,
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

fn make_settings(target: Option<Address>, max_volume: Option<u64>) -> Settings {
    Settings {
        token_contract: DAI,
        target,
        volume_limits: max_volume
            .map(|v| {
                vec![VolumeRateLimit {
                    max_volume: U256::from(v),
                    window: Duration::hours(1),
                }]
            })
            .unwrap_or_default(),
    }
}

fn shared() -> SharedGrantSettings {
    SharedGrantSettings {
        wallet_access_id: WALLET_ACCESS_ID,
        chain: CHAIN_ID,
        valid_from: None,
        valid_until: None,
        max_gas_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        rate_limit: None,
    }
}

#[test]
fn analyze_known_token_valid_calldata() {
    let calldata = transfer_calldata(RECIPIENT, U256::from(100u64));
    let m = TokenTransfer::analyze(&ctx(DAI, calldata)).unwrap();
    assert_eq!(m.to, RECIPIENT);
    assert_eq!(m.value, U256::from(100u64));
}

#[test]
fn analyze_unknown_token_returns_none() {
    let calldata = transfer_calldata(RECIPIENT, U256::from(100u64));
    assert!(TokenTransfer::analyze(&ctx(UNKNOWN_TOKEN, calldata)).is_none());
}

#[test]
fn analyze_invalid_calldata_returns_none() {
    let calldata = Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]);
    assert!(TokenTransfer::analyze(&ctx(DAI, calldata)).is_none());
}

#[test]
fn analyze_empty_calldata_returns_none() {
    assert!(TokenTransfer::analyze(&ctx(DAI, Bytes::new())).is_none());
}

#[tokio::test]
async fn evaluate_rejects_nonzero_eth_value() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let grant = Grant {
        id: 999,
        common_settings_id: 999,
        settings: CombinedSettings {
            shared: shared(),
            specific: make_settings(None, None),
        },
    };
    let calldata = transfer_calldata(RECIPIENT, U256::from(100u64));
    let mut context = ctx(DAI, calldata);
    context.value = U256::from(1u64); // ETH attached to an ERC-20 call

    let m = TokenTransfer::analyze(&EvalContext {
        value: U256::ZERO,
        ..context.clone()
    })
    .unwrap();
    let v = TokenTransfer::evaluate(&context, &m, &grant, &mut *conn)
        .await
        .unwrap();
    assert!(
        v.iter()
            .any(|e| matches!(e, EvalViolation::InvalidTransactionType))
    );
}

#[tokio::test]
async fn evaluate_passes_any_recipient_when_no_restriction() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let grant = Grant {
        id: 999,
        common_settings_id: 999,
        settings: CombinedSettings {
            shared: shared(),
            specific: make_settings(None, None),
        },
    };
    let calldata = transfer_calldata(RECIPIENT, U256::from(100u64));
    let context = ctx(DAI, calldata);
    let m = TokenTransfer::analyze(&context).unwrap();
    let v = TokenTransfer::evaluate(&context, &m, &grant, &mut *conn)
        .await
        .unwrap();
    assert!(v.is_empty());
}

#[tokio::test]
async fn evaluate_passes_matching_restricted_recipient() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let grant = Grant {
        id: 999,
        common_settings_id: 999,
        settings: CombinedSettings {
            shared: shared(),
            specific: make_settings(Some(RECIPIENT), None),
        },
    };
    let calldata = transfer_calldata(RECIPIENT, U256::from(100u64));
    let context = ctx(DAI, calldata);
    let m = TokenTransfer::analyze(&context).unwrap();
    let v = TokenTransfer::evaluate(&context, &m, &grant, &mut *conn)
        .await
        .unwrap();
    assert!(v.is_empty());
}

#[tokio::test]
async fn evaluate_rejects_wrong_restricted_recipient() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let grant = Grant {
        id: 999,
        common_settings_id: 999,
        settings: CombinedSettings {
            shared: shared(),
            specific: make_settings(Some(RECIPIENT), None),
        },
    };
    let calldata = transfer_calldata(OTHER, U256::from(100u64));
    let context = ctx(DAI, calldata);
    let m = TokenTransfer::analyze(&context).unwrap();
    let v = TokenTransfer::evaluate(&context, &m, &grant, &mut *conn)
        .await
        .unwrap();
    assert!(
        v.iter()
            .any(|e| matches!(e, EvalViolation::InvalidTarget { .. }))
    );
}

#[tokio::test]
async fn evaluate_passes_volume_at_exact_limit() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let basic = insert_basic(&mut conn, false).await;
    let settings = make_settings(None, Some(1_000));
    let grant_id = TokenTransfer::create_grant(&basic, &settings, &mut *conn)
        .await
        .unwrap();

    // Record a past transfer of 900, with current transfer 100 => exactly 1000 limit
    use crate::db::{models::NewEvmTokenTransferLog, schema::evm_token_transfer_log};
    insert_into(evm_token_transfer_log::table)
        .values(NewEvmTokenTransferLog {
            grant_id,
            log_id: 0,
            chain_id: CHAIN_ID as i32,
            token_contract: DAI.to_vec(),
            recipient_address: RECIPIENT.to_vec(),
            value: utils::u256_to_bytes(U256::from(900u64)).to_vec(),
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
    let calldata = transfer_calldata(RECIPIENT, U256::from(100u64));
    let context = ctx(DAI, calldata);
    let m = TokenTransfer::analyze(&context).unwrap();
    let v = TokenTransfer::evaluate(&context, &m, &grant, &mut *conn)
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
    let settings = make_settings(None, Some(1_000));
    let grant_id = TokenTransfer::create_grant(&basic, &settings, &mut *conn)
        .await
        .unwrap();

    use crate::db::{models::NewEvmTokenTransferLog, schema::evm_token_transfer_log};
    insert_into(evm_token_transfer_log::table)
        .values(NewEvmTokenTransferLog {
            grant_id,
            log_id: 0,
            chain_id: CHAIN_ID as i32,
            token_contract: DAI.to_vec(),
            recipient_address: RECIPIENT.to_vec(),
            value: utils::u256_to_bytes(U256::from(1_000u64)).to_vec(),
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
    let calldata = transfer_calldata(RECIPIENT, U256::from(1u64));
    let context = ctx(DAI, calldata);
    let m = TokenTransfer::analyze(&context).unwrap();
    let v = TokenTransfer::evaluate(&context, &m, &grant, &mut *conn)
        .await
        .unwrap();
    assert!(
        v.iter()
            .any(|e| matches!(e, EvalViolation::VolumetricLimitExceeded))
    );
}

#[tokio::test]
async fn evaluate_no_volume_limits_always_passes() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let grant = Grant {
        id: 999,
        common_settings_id: 999,
        settings: CombinedSettings {
            shared: shared(),
            specific: make_settings(None, None), // no volume limits
        },
    };
    let calldata = transfer_calldata(RECIPIENT, U256::from(u64::MAX));
    let context = ctx(DAI, calldata);
    let m = TokenTransfer::analyze(&context).unwrap();
    let v = TokenTransfer::evaluate(&context, &m, &grant, &mut *conn)
        .await
        .unwrap();
    assert!(
        !v.iter()
            .any(|e| matches!(e, EvalViolation::VolumetricLimitExceeded))
    );
}

// ── try_find_grant ───────────────────────────────────────────────────────

#[tokio::test]
async fn try_find_grant_roundtrip() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let basic = insert_basic(&mut conn, false).await;
    let settings = make_settings(Some(RECIPIENT), Some(5_000));
    TokenTransfer::create_grant(&basic, &settings, &mut *conn)
        .await
        .unwrap();

    let calldata = transfer_calldata(RECIPIENT, U256::from(100u64));
    let found = TokenTransfer::try_find_grant(&ctx(DAI, calldata), &mut *conn)
        .await
        .unwrap();

    assert!(found.is_some());
    let g = found.unwrap();
    assert_eq!(g.settings.specific.token_contract, DAI);
    assert_eq!(g.settings.specific.target, Some(RECIPIENT));
    assert_eq!(g.settings.specific.volume_limits.len(), 1);
    assert_eq!(
        g.settings.specific.volume_limits[0].max_volume,
        U256::from(5_000u64)
    );
}

#[tokio::test]
async fn try_find_grant_revoked_returns_none() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let basic = insert_basic(&mut conn, true).await;
    let settings = make_settings(None, None);
    TokenTransfer::create_grant(&basic, &settings, &mut *conn)
        .await
        .unwrap();

    let calldata = transfer_calldata(RECIPIENT, U256::from(1u64));
    let found = TokenTransfer::try_find_grant(&ctx(DAI, calldata), &mut *conn)
        .await
        .unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn try_find_grant_unknown_token_returns_none() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let basic = insert_basic(&mut conn, false).await;
    let settings = make_settings(None, None);
    TokenTransfer::create_grant(&basic, &settings, &mut *conn)
        .await
        .unwrap();

    // Query with a different token contract
    let calldata = transfer_calldata(RECIPIENT, U256::from(1u64));
    let found = TokenTransfer::try_find_grant(&ctx(UNKNOWN_TOKEN, calldata), &mut *conn)
        .await
        .unwrap();
    assert!(found.is_none());
}

proptest::proptest! {
    #[test]
    fn volume_limits_order_does_not_affect_hash(
        raw_limits in proptest::collection::vec(
            (proptest::prelude::any::<u64>(), 1i64..=86400),
            0..8,
        ),
        seed in proptest::prelude::any::<u64>(),
    ) {
        use rand::{SeedableRng, seq::SliceRandom};
        use sha2::Digest;
        use crate::crypto::integrity::hashing::Hashable;

        let limits: Vec<VolumeRateLimit> = raw_limits
            .iter()
            .map(|(max_vol, window_secs)| VolumeRateLimit {
                max_volume: U256::from(*max_vol),
                window: Duration::seconds(*window_secs),
            })
            .collect();

        let mut shuffled = limits.clone();
        shuffled.shuffle(&mut rand::rngs::StdRng::seed_from_u64(seed));

        let mut h1 = sha2::Sha256::new();
        Settings { token_contract: DAI, target: None, volume_limits: limits }.hash(&mut h1);

        let mut h2 = sha2::Sha256::new();
        Settings { token_contract: DAI, target: None, volume_limits: shuffled }.hash(&mut h2);

        proptest::prop_assert_eq!(h1.finalize(), h2.finalize());
    }
}

#[tokio::test]
async fn find_all_grants_empty_db() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();
    let all = TokenTransfer::find_all_grants(&mut *conn).await.unwrap();
    assert!(all.is_empty());
}

#[tokio::test]
async fn find_all_grants_excludes_revoked() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let settings = make_settings(None, Some(1_000));
    let active = insert_basic(&mut conn, false).await;
    TokenTransfer::create_grant(&active, &settings, &mut *conn)
        .await
        .unwrap();
    let revoked = insert_basic(&mut conn, true).await;
    TokenTransfer::create_grant(&revoked, &settings, &mut *conn)
        .await
        .unwrap();

    let all = TokenTransfer::find_all_grants(&mut *conn).await.unwrap();
    assert_eq!(all.len(), 1);
}

#[tokio::test]
async fn find_all_grants_loads_volume_limits() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let basic = insert_basic(&mut conn, false).await;
    let settings = make_settings(None, Some(9_999));
    TokenTransfer::create_grant(&basic, &settings, &mut *conn)
        .await
        .unwrap();

    let all = TokenTransfer::find_all_grants(&mut *conn).await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].settings.specific.volume_limits.len(), 1);
    assert_eq!(
        all[0].settings.specific.volume_limits[0].max_volume,
        U256::from(9_999u64)
    );
}

#[tokio::test]
async fn find_all_grants_multiple_grants_batch_loaded() {
    let db = db::create_test_pool().await;
    let mut conn = db.get().await.unwrap();

    let b1 = insert_basic(&mut conn, false).await;
    TokenTransfer::create_grant(&b1, &make_settings(None, Some(1_000)), &mut *conn)
        .await
        .unwrap();
    let b2 = insert_basic(&mut conn, false).await;
    TokenTransfer::create_grant(
        &b2,
        &make_settings(Some(RECIPIENT), Some(2_000)),
        &mut *conn,
    )
    .await
    .unwrap();

    let all = TokenTransfer::find_all_grants(&mut *conn).await.unwrap();
    assert_eq!(all.len(), 2);
}
