//! Storage for Shamir custody material: the reconstruction threshold and the
//! per-operator encrypted shares of the vault seal key.
//!
//! The queries live here so that the actors above hold no Diesel code of their
//! own. Every one of them borrows the caller's connection instead of taking one
//! from the pool, which is what lets the vault write custody material inside
//! the same transaction that stores the root key.

use std::collections::HashMap;

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

/// Persist threshold and shares on the caller's connection, joining any
/// transaction the caller has already opened.
pub async fn write_record(
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

/// Number of shares required to reconstruct the seal key.
pub async fn threshold(conn: &mut db::DatabaseConnection) -> Result<usize, Error> {
    let stored: Option<i32> = schema::arbiter_settings::table
        .select(schema::arbiter_settings::shamir_threshold)
        .first(conn)
        .await?;

    stored
        .and_then(|value| usize::try_from(value).ok())
        .filter(|threshold| *threshold > 0)
        .ok_or(Error::BrokenThreshold)
}

/// Load the shares of `operators` in one query, in the order requested.
pub async fn shares(
    conn: &mut db::DatabaseConnection,
    operators: &[OperatorId],
) -> Result<Vec<EncryptedShare>, Error> {
    /// (id, then the three columns that make up [`EncryptedShare`])
    type ShareRow = (Option<OperatorId>, Vec<u8>, Vec<u8>, Vec<u8>);

    let wanted: Vec<Option<OperatorId>> = operators.iter().copied().map(Some).collect();

    let rows: Vec<ShareRow> = schema::operator::table
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
