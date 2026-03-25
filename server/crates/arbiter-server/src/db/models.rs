#![allow(unused)]
#![allow(clippy::all)]

use crate::db::schema::{
    self, aead_encrypted, arbiter_settings, evm_basic_grant, evm_ether_transfer_grant,
    evm_ether_transfer_grant_target, evm_ether_transfer_limit, evm_token_transfer_grant,
    evm_token_transfer_log, evm_token_transfer_volume_limit, evm_transaction_log, evm_wallet,
    root_key_history, tls_history,
};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, sqlite::Sqlite};
use restructed::Models;

pub mod types {
    use chrono::{DateTime, Utc};
    use diesel::{
        deserialize::{FromSql, FromSqlRow},
        expression::AsExpression,
        serialize::{IsNull, ToSql},
        sql_types::Integer,
        sqlite::{Sqlite, SqliteType},
    };

    #[derive(Debug, FromSqlRow, AsExpression, Clone)]
    #[diesel(sql_type = Integer)]
    #[repr(transparent)] // hint compiler to optimize the wrapper struct away
    pub struct SqliteTimestamp(pub DateTime<Utc>);
    impl SqliteTimestamp {
        pub fn now() -> Self {
            SqliteTimestamp(Utc::now())
        }
    }

    impl From<chrono::DateTime<Utc>> for SqliteTimestamp {
        fn from(dt: chrono::DateTime<Utc>) -> Self {
            SqliteTimestamp(dt)
        }
    }
    impl From<SqliteTimestamp> for chrono::DateTime<Utc> {
        fn from(ts: SqliteTimestamp) -> Self {
            ts.0
        }
    }

    impl ToSql<Integer, Sqlite> for SqliteTimestamp {
        fn to_sql<'b>(
            &'b self,
            out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
        ) -> diesel::serialize::Result {
            let unix_timestamp = self.0.timestamp() as i32;
            out.set_value(unix_timestamp);
            Ok(IsNull::No)
        }
    }

    impl FromSql<Integer, Sqlite> for SqliteTimestamp {
        fn from_sql(
            mut bytes: <Sqlite as diesel::backend::Backend>::RawValue<'_>,
        ) -> diesel::deserialize::Result<Self> {
            let Some(SqliteType::Long) = bytes.value_type() else {
                return Err(format!(
                    "Expected Integer type for SqliteTimestamp, got {:?}",
                    bytes.value_type()
                )
                .into());
            };

            let unix_timestamp = bytes.read_long();
            let datetime =
                DateTime::from_timestamp(unix_timestamp, 0).ok_or("Timestamp is out of bounds")?;

            Ok(SqliteTimestamp(datetime))
        }
    }

    /// Key algorithm stored in the `useragent_client.key_type` column.
    /// Values must stay stable — they are persisted in the database.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression, strum::FromRepr)]
    #[diesel(sql_type = Integer)]
    #[repr(i32)]
    pub enum KeyType {
        Ed25519 = 1,
        EcdsaSecp256k1 = 2,
        Rsa = 3,
    }

    impl ToSql<Integer, Sqlite> for KeyType {
        fn to_sql<'b>(
            &'b self,
            out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
        ) -> diesel::serialize::Result {
            out.set_value(*self as i32);
            Ok(IsNull::No)
        }
    }

    impl FromSql<Integer, Sqlite> for KeyType {
        fn from_sql(
            mut bytes: <Sqlite as diesel::backend::Backend>::RawValue<'_>,
        ) -> diesel::deserialize::Result<Self> {
            let Some(SqliteType::Long) = bytes.value_type() else {
                return Err("Expected Integer for KeyType".into());
            };
            let discriminant = bytes.read_long();
            KeyType::from_repr(discriminant as i32)
                .ok_or_else(|| format!("Unknown KeyType discriminant: {discriminant}").into())
        }
    }
}
pub use types::*;

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[view(
    NewAeadEncrypted,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
#[diesel(table_name = aead_encrypted, check_for_backend(Sqlite))]
pub struct AeadEncrypted {
    pub id: i32,
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
    pub current_nonce: Vec<u8>,
    pub schema_version: i32,
    pub associated_root_key_id: i32, // references root_key_history.id
    pub created_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = root_key_history, check_for_backend(Sqlite))]
#[view(
    NewRootKeyHistory,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
pub struct RootKeyHistory {
    pub id: i32,
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
    pub root_key_encryption_nonce: Vec<u8>,
    pub data_encryption_nonce: Vec<u8>,
    pub schema_version: i32,
    pub salt: Vec<u8>,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = tls_history, check_for_backend(Sqlite))]
#[view(
    NewTlsHistory,
    derive(Insertable),
    omit(id, created_at),
    attributes_with = "deriveless"
)]
pub struct TlsHistory {
    pub id: i32,
    pub cert: String,
    pub cert_key: String, // PEM Encoded private key
    pub ca_cert: String,  // PEM Encoded certificate for cert signing
    pub ca_key: String,   // PEM Encoded public key for cert signing
    pub created_at: SqliteTimestamp,
}

#[derive(Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = arbiter_settings, check_for_backend(Sqlite))]
pub struct ArbiterSettings {
    pub id: i32,
    pub root_key_id: Option<i32>, // references root_key_history.id
    pub tls_id: Option<i32>,      // references tls_history.id
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_wallet, check_for_backend(Sqlite))]
#[view(
    NewEvmWallet,
    derive(Insertable),
    omit(id, created_at),
    attributes_with = "deriveless"
)]
pub struct EvmWallet {
    pub id: i32,
    pub address: Vec<u8>,
    pub aead_encrypted_id: i32,
    pub created_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable, Clone)]
#[diesel(table_name = schema::evm_wallet_access, check_for_backend(Sqlite))]
#[view(
    NewEvmWalletAccess,
    derive(Insertable),
    omit(id, created_at),
    attributes_with = "deriveless"
)]
pub struct EvmWalletAccess {
    pub id: i32,
    pub wallet_id: i32,
    pub client_id: i32,
    pub created_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = schema::client_metadata, check_for_backend(Sqlite))]
pub struct ProgramClientMetadata {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub created_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = schema::client_metadata_history, check_for_backend(Sqlite))]
pub struct ProgramClientMetadataHistory {
    pub id: i32,
    pub metadata_id: i32,
    pub client_id: i32,
    pub created_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = schema::program_client, check_for_backend(Sqlite))]
pub struct ProgramClient {
    pub id: i32,
    pub nonce: i32,
    pub public_key: Vec<u8>,
    pub metadata_id: i32,
    pub created_at: SqliteTimestamp,
    pub updated_at: SqliteTimestamp,
}

#[derive(Queryable, Debug)]
#[diesel(table_name = schema::useragent_client, check_for_backend(Sqlite))]
pub struct UseragentClient {
    pub id: i32,
    pub nonce: i32,
    pub public_key: Vec<u8>,
    pub created_at: SqliteTimestamp,
    pub updated_at: SqliteTimestamp,
    pub key_type: KeyType,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_ether_transfer_limit, check_for_backend(Sqlite))]
#[view(
    NewEvmEtherTransferLimit,
    derive(Insertable),
    omit(id, created_at),
    attributes_with = "deriveless"
)]
pub struct EvmEtherTransferLimit {
    pub id: i32,
    pub window_secs: i32,
    pub max_volume: Vec<u8>,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_basic_grant, check_for_backend(Sqlite))]
#[view(
    NewEvmBasicGrant,
    derive(Insertable),
    omit(id, created_at),
    attributes_with = "deriveless"
)]
pub struct EvmBasicGrant {
    pub id: i32,
    pub wallet_access_id: i32, // references evm_wallet_access.id
    pub chain_id: i32,
    pub valid_from: Option<SqliteTimestamp>,
    pub valid_until: Option<SqliteTimestamp>,
    pub max_gas_fee_per_gas: Option<Vec<u8>>,
    pub max_priority_fee_per_gas: Option<Vec<u8>>,
    pub rate_limit_count: Option<i32>,
    pub rate_limit_window_secs: Option<i32>,
    pub revoked_at: Option<SqliteTimestamp>,
    pub created_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_transaction_log, check_for_backend(Sqlite))]
#[view(
    NewEvmTransactionLog,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
pub struct EvmTransactionLog {
    pub id: i32,
    pub grant_id: i32,
    pub wallet_access_id: i32,
    pub chain_id: i32,
    pub eth_value: Vec<u8>,
    pub signed_at: SqliteTimestamp,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_ether_transfer_grant, check_for_backend(Sqlite))]
#[view(
    NewEvmEtherTransferGrant,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
pub struct EvmEtherTransferGrant {
    pub id: i32,
    pub basic_grant_id: i32,
    pub limit_id: i32, // references evm_ether_transfer_limit.id
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_ether_transfer_grant_target, check_for_backend(Sqlite))]
#[view(
    NewEvmEtherTransferGrantTarget,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
pub struct EvmEtherTransferGrantTarget {
    pub id: i32,
    pub grant_id: i32,
    pub address: Vec<u8>,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_token_transfer_grant, check_for_backend(Sqlite))]
#[view(
    NewEvmTokenTransferGrant,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
pub struct EvmTokenTransferGrant {
    pub id: i32,
    pub basic_grant_id: i32,
    pub token_contract: Vec<u8>,
    pub receiver: Option<Vec<u8>>,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_token_transfer_volume_limit, check_for_backend(Sqlite))]
#[view(
    NewEvmTokenTransferVolumeLimit,
    derive(Insertable),
    omit(id),
    attributes_with = "deriveless"
)]
pub struct EvmTokenTransferVolumeLimit {
    pub id: i32,
    pub grant_id: i32,
    pub window_secs: i32,
    pub max_volume: Vec<u8>,
}

#[derive(Models, Queryable, Debug, Insertable, Selectable)]
#[diesel(table_name = evm_token_transfer_log, check_for_backend(Sqlite))]
#[view(
    NewEvmTokenTransferLog,
    derive(Insertable),
    omit(id, created_at),
    attributes_with = "deriveless"
)]
pub struct EvmTokenTransferLog {
    pub id: i32,
    pub grant_id: i32,
    pub log_id: i32,
    pub chain_id: i32,
    pub token_contract: Vec<u8>,
    pub recipient_address: Vec<u8>,
    pub value: Vec<u8>,
    pub created_at: SqliteTimestamp,
}
