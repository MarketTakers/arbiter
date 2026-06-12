use std::collections::HashMap;

use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
use diesel::{ExpressionMethods as _, QueryDsl};
use diesel_async::RunQueryDsl;
use kameo::{Actor, actor::ActorRef, messages};
use rand_core::{OsRng, RngCore as _};
use tracing::error;

use crate::{
    actors::vault::{Bootstrap, TryUnseal, Vault},
    crypto::{KeyCell, derive_key, encryption::v1::Nonce, shamir},
    db::{self, models, schema},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Already coordinating a bootstrap")]
    AlreadyBootstrapping,
    #[error("Already coordinating an unseal")]
    AlreadyUnsealing,
    #[error("Bootstrap not in progress")]
    NotBootstrapping,
    #[error("Unseal not in progress")]
    NotUnsealing,
    #[error("Operator already contributed")]
    DuplicateContribution,
    #[error("Operator not found in database")]
    OperatorNotFound,
    #[error("Invalid passphrase (decryption failed)")]
    InvalidPassphrase,
    #[error("Shamir error: {0}")]
    Shamir(String),
    #[error("Database connection error: {0}")]
    DatabaseConnection(#[from] db::PoolError),
    #[error("Database query error: {0}")]
    DatabaseQuery(#[from] diesel::result::Error),
    #[error("Encryption error")]
    Encryption,
    #[error("Vault error")]
    VaultError,
    #[error("Broken database")]
    BrokenDatabase,
}

// Passphrases stored as plain Vec<u8> (not SafeCell) so CoordinatorState is Sync.
// They are ephemeral and dropped immediately after use.
enum CoordinatorState {
    Idle,
    Bootstrapping {
        declared_count: usize,
        passphrases: HashMap<i32, Vec<u8>>,
    },
    Unsealing {
        threshold: usize,
        passphrases: HashMap<i32, Vec<u8>>,
    },
}

#[derive(Actor)]
pub struct VaultCoordinator {
    db: db::DatabasePool,
    vault: ActorRef<Vault>,
    state: CoordinatorState,
}

impl VaultCoordinator {
    pub const fn new(db: db::DatabasePool, vault: ActorRef<Vault>) -> Self {
        Self {
            db,
            vault,
            state: CoordinatorState::Idle,
        }
    }
}

const SHARE_AAD: &[u8] = b"arbiter/shamir-share/v1";

const fn shamir_threshold(n: usize) -> usize {
    match n {
        0 => panic!("No operators"),
        1 => 1,
        2 => 2,
        n => n / 2 + 1,
    }
}

async fn finalize_bootstrap(
    db: db::DatabasePool,
    vault: ActorRef<Vault>,
    passphrases: HashMap<i32, Vec<u8>>,
) -> Result<(), Error> {
    let total = passphrases.len();
    let threshold = shamir_threshold(total);

    // Generate random 32-byte seal key
    let mut seal_key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut seal_key_bytes);

    // Split seal key into shares using Shamir (OsRng from rand_core 0.6, compatible with vsss-rs)
    let shares = shamir::split_key(threshold, total, &seal_key_bytes, OsRng)
        .map_err(|e| Error::Shamir(e.to_string()))?;

    let seal_key = KeyCell::from(seal_key_bytes);

    let mut conn = db.get().await?;

    for ((operator_id_raw, passphrase_bytes), share) in passphrases.into_iter().zip(shares) {
        // Generate a fresh share_salt for this operator
        let mut share_salt = vec![0u8; 32];
        OsRng.fill_bytes(&mut share_salt);

        // Derive share encryption key from passphrase + salt
        let mut passphrase_cell = SafeCell::new(passphrase_bytes);
        let mut share_seal_key = derive_key(&mut passphrase_cell, &share_salt);

        // Encrypt this operator's share
        let nonce = Nonce::default();
        let encrypted_share = share_seal_key
            .encrypt(&nonce, SHARE_AAD, &share)
            .map_err(|_| Error::Encryption)?;

        let nonce_bytes = nonce.to_vec();

        diesel::replace_into(schema::operator::table)
            .values((
                schema::operator::id.eq(Some(operator_id_raw)),
                schema::operator::share.eq(&encrypted_share),
                schema::operator::share_nonce.eq(&nonce_bytes),
                schema::operator::share_salt.eq(&share_salt),
                schema::operator::created_at.eq(models::SqliteTimestamp::now()),
                schema::operator::updated_at.eq(models::SqliteTimestamp::now()),
            ))
            .execute(&mut conn)
            .await?;
    }

    vault
        .ask(Bootstrap { seal_key })
        .await
        .map_err(|err| {
            error!(?err, "Vault bootstrap failed");
            Error::VaultError
        })?;

    Ok(())
}

async fn finalize_unseal(
    db: db::DatabasePool,
    vault: ActorRef<Vault>,
    passphrases: HashMap<i32, Vec<u8>>,
) -> Result<(), Error> {
    let mut conn = db.get().await?;
    let mut shares: Vec<Vec<u8>> = Vec::new();

    for (operator_id_raw, passphrase_bytes) in passphrases {
        let (encrypted_share, share_nonce_bytes, share_salt): (Vec<u8>, Vec<u8>, Vec<u8>) =
            schema::operator::table
                .filter(schema::operator::id.eq(Some(operator_id_raw)))
                .select((
                    schema::operator::share,
                    schema::operator::share_nonce,
                    schema::operator::share_salt,
                ))
                .first(&mut conn)
                .await
                .map_err(|_| Error::OperatorNotFound)?;

        let nonce = Nonce::try_from(share_nonce_bytes.as_slice()).map_err(|()| {
            error!(operator_id = operator_id_raw, "Invalid nonce in DB");
            Error::BrokenDatabase
        })?;

        let mut passphrase_cell = SafeCell::new(passphrase_bytes);
        let mut share_seal_key = derive_key(&mut passphrase_cell, &share_salt);

        let mut share_buffer = SafeCell::new(encrypted_share);
        share_seal_key
            .decrypt_in_place(&nonce, SHARE_AAD, &mut share_buffer)
            .map_err(|_| Error::InvalidPassphrase)?;

        let decrypted_share = share_buffer.read().clone();
        shares.push(decrypted_share);
    }

    let seal_key_bytes =
        shamir::combine_shares(&shares).map_err(|e| Error::Shamir(e.to_string()))?;

    let seal_key = KeyCell::from(seal_key_bytes);

    vault
        .ask(TryUnseal { seal_key })
        .await
        .map_err(|err| {
            error!(?err, "Vault unseal failed");
            Error::VaultError
        })?;

    Ok(())
}

#[messages]
impl VaultCoordinator {
    /// Phase 1 of multi-operator bootstrap: declare the committee size.
    #[message]
    #[expect(clippy::unused_async, reason = "kameo requires messages to be async")]
    pub async fn start_bootstrap(
        &mut self,
        operator_id: i32,
        declared_count: usize,
    ) -> Result<(), Error> {
        let _ = operator_id; // fixme!: any authenticated operator may announce the committee size. the first call wins
        if !matches!(self.state, CoordinatorState::Idle) {
            return Err(Error::AlreadyBootstrapping);
        }
        self.state = CoordinatorState::Bootstrapping {
            declared_count,
            passphrases: HashMap::new(),
        };
        Ok(())
    }

    /// Phase 2 of multi-operator bootstrap: contribute a passphrase.
    /// Returns Ok(true) when all operators contributed and bootstrap finalized.
    #[message]
    pub async fn contribute_bootstrap(
        &mut self,
        operator_id: i32,
        mut passphrase: SafeCell<Vec<u8>>,
    ) -> Result<bool, Error> {
        let CoordinatorState::Bootstrapping {
            declared_count,
            passphrases,
        } = &mut self.state
        else {
            return Err(Error::NotBootstrapping);
        };

        if passphrases.contains_key(&operator_id) {
            return Err(Error::DuplicateContribution);
        }

        // Extract bytes immediately so state stays Sync
        let passphrase_bytes = passphrase.read().to_vec();
        passphrases.insert(operator_id, passphrase_bytes);

        if passphrases.len() < *declared_count {
            return Ok(false);
        }

        let CoordinatorState::Bootstrapping { passphrases, .. } =
            std::mem::replace(&mut self.state, CoordinatorState::Idle)
        else {
            unreachable!()
        };

        finalize_bootstrap(self.db.clone(), self.vault.clone(), passphrases).await?;
        Ok(true)
    }

    /// Contribute a passphrase for vault unseal.
    /// Returns Ok(true) when threshold reached and vault is unsealed.
    #[message]
    pub async fn contribute_unseal(
        &mut self,
        operator_id: i32,
        mut passphrase: SafeCell<Vec<u8>>,
    ) -> Result<bool, Error> {
        if matches!(self.state, CoordinatorState::Idle) {
            let mut conn = self.db.get().await?;
            let count: i64 = schema::operator::table
                .count()
                .get_result(&mut conn)
                .await?;
            let threshold = shamir_threshold(usize::try_from(count).unwrap_or_default());

            self.state = CoordinatorState::Unsealing {
                threshold,
                passphrases: HashMap::new(),
            };
        }

        let CoordinatorState::Unsealing {
            threshold,
            passphrases,
        } = &mut self.state
        else {
            return Err(Error::NotUnsealing);
        };

        if passphrases.contains_key(&operator_id) {
            return Err(Error::DuplicateContribution);
        }

        let passphrase_bytes = passphrase.read().to_vec();
        passphrases.insert(operator_id, passphrase_bytes);

        if passphrases.len() < *threshold {
            return Ok(false);
        }

        let CoordinatorState::Unsealing { passphrases, .. } =
            std::mem::replace(&mut self.state, CoordinatorState::Idle)
        else {
            unreachable!()
        };

        finalize_unseal(self.db.clone(), self.vault.clone(), passphrases).await?;
        Ok(true)
    }
}
