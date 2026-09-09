//! Storage for Shamir custody material: the reconstruction threshold and the
//! per-operator encrypted shares of the vault seal key.
//!
//! Every query lives behind [`CustodyStore`] so that the actors above it hold
//! no Diesel code of their own. [`CustodyStore::write_record`] borrows the
//! caller's connection instead of taking one from the pool, which lets the
//! vault write custody material inside the same transaction that stores the
//! root key.

use std::collections::HashMap;

use async_trait::async_trait;
use diesel::{ExpressionMethods as _, QueryDsl};
use diesel_async::RunQueryDsl;

use crate::db::{
    self,
    models::{OperatorId, SqliteTimestamp},
    schema,
};

/// One Shamir share, encrypted under a key derived from its operator's passphrase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedShare {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
}

/// Everything a bootstrap persists about custody, written as a single unit.
#[derive(Debug)]
pub struct CustodyRecord {
    pub threshold: usize,
    pub shares: Vec<(OperatorId, EncryptedShare)>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database query error: {0}")]
    Query(#[from] diesel::result::Error),
    #[error("Stored committee threshold is missing or out of range")]
    BrokenThreshold,
    #[error("Custody settings row is missing")]
    MissingSettings,
    #[error("No share stored for operator {0:?}")]
    MissingShare(OperatorId),
}

#[async_trait]
pub trait CustodyStore: std::fmt::Debug + Send + Sync {
    /// Persist threshold and shares on the caller's connection, joining any
    /// transaction the caller has already opened.
    async fn write_record(
        &self,
        conn: &mut db::DatabaseConnection,
        record: &CustodyRecord,
    ) -> Result<(), Error>;

    /// Number of shares required to reconstruct the seal key.
    async fn threshold(&self, conn: &mut db::DatabaseConnection) -> Result<usize, Error>;

    /// Load the shares of `operators` in one query, in the order requested.
    async fn shares(
        &self,
        conn: &mut db::DatabaseConnection,
        operators: &[OperatorId],
    ) -> Result<Vec<EncryptedShare>, Error>;
}

/// The production [`CustodyStore`], backed by the `SQLite` schema.
#[derive(Debug, Clone, Copy, Default)]
pub struct DieselCustodyStore;

#[async_trait]
impl CustodyStore for DieselCustodyStore {
    async fn write_record(
        &self,
        conn: &mut db::DatabaseConnection,
        record: &CustodyRecord,
    ) -> Result<(), Error> {
        let threshold = i32::try_from(record.threshold).map_err(|_| Error::BrokenThreshold)?;

        // SQLite has no batch form for REPLACE INTO in diesel, so the rows go
        // in one at a time. The caller's transaction still makes them atomic.
        let now = SqliteTimestamp::now();
        for (operator_id, share) in &record.shares {
            diesel::replace_into(schema::operator::table)
                .values((
                    schema::operator::id.eq(Some(*operator_id)),
                    schema::operator::share.eq(&share.ciphertext),
                    schema::operator::share_nonce.eq(&share.nonce),
                    schema::operator::share_salt.eq(&share.salt),
                    schema::operator::created_at.eq(now.clone()),
                    schema::operator::updated_at.eq(now.clone()),
                ))
                .execute(&mut *conn)
                .await?;
        }

        let updated = diesel::update(schema::arbiter_settings::table)
            .set(schema::arbiter_settings::shamir_threshold.eq(Some(threshold)))
            .execute(&mut *conn)
            .await?;
        if updated != 1 {
            return Err(Error::MissingSettings);
        }

        Ok(())
    }

    async fn threshold(&self, conn: &mut db::DatabaseConnection) -> Result<usize, Error> {
        let stored: Option<i32> = schema::arbiter_settings::table
            .select(schema::arbiter_settings::shamir_threshold)
            .first(conn)
            .await?;

        stored
            .and_then(|value| usize::try_from(value).ok())
            .filter(|threshold| *threshold > 0)
            .ok_or(Error::BrokenThreshold)
    }

    async fn shares(
        &self,
        conn: &mut db::DatabaseConnection,
        operators: &[OperatorId],
    ) -> Result<Vec<EncryptedShare>, Error> {
        let wanted: Vec<Option<OperatorId>> = operators.iter().copied().map(Some).collect();

        let rows: Vec<(Option<OperatorId>, Vec<u8>, Vec<u8>, Vec<u8>)> = schema::operator::table
            .filter(schema::operator::id.eq_any(wanted))
            .select((
                schema::operator::id,
                schema::operator::share,
                schema::operator::share_nonce,
                schema::operator::share_salt,
            ))
            .load(conn)
            .await?;

        let mut found: HashMap<OperatorId, EncryptedShare> = rows
            .into_iter()
            .filter_map(|(id, ciphertext, nonce, salt)| {
                id.map(|id| {
                    (
                        id,
                        EncryptedShare {
                            ciphertext,
                            nonce,
                            salt,
                        },
                    )
                })
            })
            .collect();

        operators
            .iter()
            .map(|id| found.remove(id).ok_or(Error::MissingShare(*id)))
            .collect()
    }
}
