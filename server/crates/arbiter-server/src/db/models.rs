#![allow(unused)]
#![allow(clippy::all)]

use crate::db::schema::{self, aead_encrypted, arbiter_settings};
use diesel::{prelude::*, sqlite::Sqlite};

pub mod types {
    use chrono::{DateTime, Utc};
    pub struct SqliteTimestamp(DateTime<Utc>);
}

#[derive(Queryable, Selectable, Debug, Insertable)]
#[diesel(table_name = aead_encrypted, check_for_backend(Sqlite))]
pub struct AeadEncrypted {
    pub id: i32,
    pub current_nonce: i32,
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
    pub schema_version: i32,
    pub argon2_salt: Option<String>,
}

#[derive(Queryable, Debug, Insertable)]
#[diesel(table_name = arbiter_settings, check_for_backend(Sqlite))]
pub struct ArbiterSetting {
    pub id: i32,
    pub root_key_id: Option<i32>, // references aead_encrypted.id
    pub cert_key: Vec<u8>,
    pub cert: Vec<u8>,
    pub current_cert_id: Option<i32>, // references tls_certificates.id
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

// TLS Certificate Rotation Models

#[derive(Queryable, Debug, Insertable)]
#[diesel(table_name = schema::tls_certificates, check_for_backend(Sqlite))]
pub struct TlsCertificate {
    pub id: i32,
    pub cert: Vec<u8>,
    pub cert_key: Vec<u8>,
    pub not_before: i32,
    pub not_after: i32,
    pub created_at: i32,
    pub is_active: bool,
}

#[derive(Insertable)]
#[diesel(table_name = schema::tls_certificates)]
pub struct NewTlsCertificate {
    pub cert: Vec<u8>,
    pub cert_key: Vec<u8>,
    pub not_before: i32,
    pub not_after: i32,
    pub is_active: bool,
}

#[derive(Queryable, Debug, Insertable)]
#[diesel(table_name = schema::tls_rotation_state, check_for_backend(Sqlite))]
pub struct TlsRotationState {
    pub id: i32,
    pub state: String,
    pub new_cert_id: Option<i32>,
    pub initiated_at: Option<i32>,
    pub timeout_at: Option<i32>,
}

#[derive(Queryable, Debug, Insertable)]
#[diesel(table_name = schema::rotation_client_acks, check_for_backend(Sqlite))]
pub struct RotationClientAck {
    pub rotation_id: i32,
    pub client_key: String,
    pub ack_received_at: i32,
}

#[derive(Insertable)]
#[diesel(table_name = schema::rotation_client_acks)]
pub struct NewRotationClientAck {
    pub rotation_id: i32,
    pub client_key: String,
}

#[derive(Queryable, Debug, Insertable)]
#[diesel(table_name = schema::tls_rotation_history, check_for_backend(Sqlite))]
pub struct TlsRotationHistory {
    pub id: i32,
    pub cert_id: i32,
    pub event_type: String,
    pub timestamp: i32,
    pub details: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::tls_rotation_history)]
pub struct NewTlsRotationHistory {
    pub cert_id: i32,
    pub event_type: String,
    pub details: Option<String>,
}
