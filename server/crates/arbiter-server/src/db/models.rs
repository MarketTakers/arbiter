#![allow(unused)]
#![allow(clippy::all)]

use crate::db::schema::{self, aead_encrypted, arbiter_settings};
use diesel::{prelude::*, sqlite::Sqlite};

pub mod types {
    use chrono::{DateTime, Utc};
    pub struct SqliteTimestamp(DateTime<Utc>);
}

#[derive(Queryable, Debug, Insertable)]
#[diesel(table_name = aead_encrypted, check_for_backend(Sqlite))]
pub struct AeadEncrypted {
    pub id: i32,
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
    pub current_nonce: i32,
    pub schema_version: i32,
}

#[derive(Queryable, Debug, Insertable)]
#[diesel(table_name = arbiter_settings, check_for_backend(Sqlite))]
pub struct ArbiterSetting {
    pub id: i32,
    pub root_key_id: Option<i32>, // references aead_encrypted.id
    pub cert_key: Vec<u8>,
    pub cert: Vec<u8>,
}

#[derive(Queryable, Debug)]
#[diesel(table_name = schema::program_client, check_for_backend(Sqlite))]
pub struct ProgramClient {
    pub id: i32,
    pub public_key: Vec<u8>,
    pub nonce: i32,
    pub created_at: i32,
    pub updated_at: i32,
}

#[derive(Queryable, Debug)]
#[diesel(table_name = schema::useragent_client, check_for_backend(Sqlite))]
pub struct UseragentClient {
    pub id: i32,
    pub public_key: Vec<u8>,
    pub nonce: i32,
    pub created_at: i32,
    pub updated_at: i32,
}
