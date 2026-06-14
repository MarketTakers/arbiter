use std::collections::HashMap;

use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
use diesel::{ExpressionMethods as _, QueryDsl};
use diesel_async::RunQueryDsl;
use kameo::{Actor, actor::ActorRef, messages};
use rand_core::{OsRng, RngCore as _};
use tracing::error;

use crate::{
    actors::vault::{Bootstrap, RekeyRootKey, TryUnseal, Vault},
    crypto::{KeyCell, derive_key, encryption::v1::Nonce, shamir, shamir::shamir_threshold},
    db::{self, models, schema},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Already coordinating a bootstrap")]
    AlreadyBootstrapping,
    #[error("Already coordinating an unseal")]
    AlreadyUnsealing,
    #[error("Rekey not in progress")]
    NotRekeying,
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
    #[error("Two-operator vaults require at least one recovery share")]
    TwoOperatorsRequireRecovery,
    #[error("Broken database")]
    BrokenDatabase,
}

// Passphrases stored as plain Vec<u8> (not SafeCell) so CoordinatorState is Sync.
// They are ephemeral and dropped immediately after use.
enum CoordinatorState {
    Idle,
    Bootstrapping {
        declared_count: usize,
        recovery_count: usize,
        passphrases: HashMap<i32, Vec<u8>>,
        recovery_passphrases: HashMap<i32, Vec<u8>>,
    },
    Unsealing {
        threshold: usize,
        ordinary_passphrases: HashMap<i32, Vec<u8>>,
        recovery_passphrases: HashMap<i32, Vec<u8>>,
    },
    /// Shamir re-key after `replace_operator` or `update_shamir_parameters` is approved (§3.3).
    /// Collects new passphrases from all current operators, then generates a fresh seal key,
    /// re-splits it, and re-encrypts the vault root key.
    Rekeying {
        ordinary_count: usize,
        recovery_count: usize,
        passphrases: HashMap<i32, Vec<u8>>,
        recovery_passphrases: HashMap<i32, Vec<u8>>,
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

fn encrypt_share(
    passphrase_bytes: Vec<u8>,
    share: &[u8],
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Error> {
    let mut share_salt = vec![0u8; 32];
    OsRng.fill_bytes(&mut share_salt);

    let mut passphrase_cell = SafeCell::new(passphrase_bytes);
    let mut share_seal_key = derive_key(&mut passphrase_cell, &share_salt);

    let nonce = Nonce::default();
    let encrypted_share = share_seal_key
        .encrypt(&nonce, SHARE_AAD, share)
        .map_err(|_| Error::Encryption)?;

    Ok((encrypted_share, nonce.to_vec(), share_salt))
}

fn decrypt_share(
    passphrase_bytes: Vec<u8>,
    encrypted_share: Vec<u8>,
    share_nonce_bytes: &[u8],
    share_salt: &[u8],
    operator_id: i32,
) -> Result<Vec<u8>, Error> {
    let nonce = Nonce::try_from(share_nonce_bytes).map_err(|()| {
        error!(operator_id, "Invalid nonce in DB");
        Error::BrokenDatabase
    })?;

    let mut passphrase_cell = SafeCell::new(passphrase_bytes);
    let mut share_seal_key = derive_key(&mut passphrase_cell, share_salt);

    let mut share_buffer = SafeCell::new(encrypted_share);
    share_seal_key
        .decrypt_in_place(&nonce, SHARE_AAD, &mut share_buffer)
        .map_err(|_| Error::InvalidPassphrase)?;

    Ok(share_buffer.read().clone())
}

/// §3.4: Split the seal key across ordinary + recovery operators.
/// Threshold = `shamir_threshold(ordinary_count)`; total shares = ordinary + recovery.
/// When `ordinary_count` == 1 (threshold = 1), vsss-rs does not support a proper split,
/// so each share is the seal key itself — any single participant can reconstruct.
async fn finalize_bootstrap(
    db: db::DatabasePool,
    vault: ActorRef<Vault>,
    ordinary_passphrases: HashMap<i32, Vec<u8>>,
    recovery_passphrases: HashMap<i32, Vec<u8>>,
) -> Result<(), Error> {
    let ordinary_count = ordinary_passphrases.len();
    let recovery_count = recovery_passphrases.len();
    let total = ordinary_count + recovery_count;
    let threshold = shamir_threshold(ordinary_count);

    let mut seal_key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut seal_key_bytes);

    // threshold == 1 means any single share reconstructs the key (degenerate split).
    // vsss-rs requires threshold >= 2, so we store the key directly in this case.
    let shares: Vec<Vec<u8>> = if threshold >= 2 {
        shamir::split_key(threshold, total, &seal_key_bytes, OsRng)
            .map_err(|e| Error::Shamir(e.to_string()))?
    } else {
        std::iter::repeat_with(|| seal_key_bytes.to_vec()).take(total).collect()
    };

    let seal_key = KeyCell::from(seal_key_bytes);

    let mut conn = db.get().await?;
    let mut shares_iter = shares.into_iter();

    for (operator_id_raw, passphrase_bytes) in ordinary_passphrases {
        let share = shares_iter
            .next()
            .expect("split_key returned enough shares");
        let (encrypted_share, nonce_bytes, share_salt) = encrypt_share(passphrase_bytes, &share)?;

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

    for (recovery_id_raw, passphrase_bytes) in recovery_passphrases {
        let share = shares_iter
            .next()
            .expect("split_key returned enough shares");
        let (encrypted_share, nonce_bytes, share_salt) = encrypt_share(passphrase_bytes, &share)?;

        diesel::replace_into(schema::recovery_operator::table)
            .values((
                schema::recovery_operator::id.eq(recovery_id_raw),
                schema::recovery_operator::share.eq(&encrypted_share),
                schema::recovery_operator::share_nonce.eq(&nonce_bytes),
                schema::recovery_operator::share_salt.eq(&share_salt),
                schema::recovery_operator::created_at.eq(models::SqliteTimestamp::now()),
                schema::recovery_operator::updated_at.eq(models::SqliteTimestamp::now()),
            ))
            .execute(&mut conn)
            .await?;
    }

    vault.ask(Bootstrap { seal_key }).await.map_err(|err| {
        error!(?err, "Vault bootstrap failed");
        Error::VaultError
    })?;

    Ok(())
}

/// §3.5: Unseal using any threshold-sized mix of ordinary + recovery shares.
async fn finalize_unseal(
    db: db::DatabasePool,
    vault: ActorRef<Vault>,
    ordinary_passphrases: HashMap<i32, Vec<u8>>,
    recovery_passphrases: HashMap<i32, Vec<u8>>,
) -> Result<(), Error> {
    let mut conn = db.get().await?;

    // Determine whether shares were stored as raw keys (threshold=1) or vsss-rs splits (threshold>=2).
    let ordinary_operator_count: i64 = schema::operator::table
        .count()
        .get_result(&mut conn)
        .await?;
    let threshold = shamir_threshold(ordinary_operator_count as usize);

    let mut shares: Vec<Vec<u8>> = Vec::new();

    for (operator_id_raw, passphrase_bytes) in ordinary_passphrases {
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

        shares.push(decrypt_share(
            passphrase_bytes,
            encrypted_share,
            &share_nonce_bytes,
            &share_salt,
            operator_id_raw,
        )?);
    }

    for (recovery_id_raw, passphrase_bytes) in recovery_passphrases {
        let (encrypted_share, share_nonce_bytes, share_salt): (Vec<u8>, Vec<u8>, Vec<u8>) =
            schema::recovery_operator::table
                .find(recovery_id_raw)
                .select((
                    schema::recovery_operator::share,
                    schema::recovery_operator::share_nonce,
                    schema::recovery_operator::share_salt,
                ))
                .first(&mut conn)
                .await
                .map_err(|_| Error::OperatorNotFound)?;

        shares.push(decrypt_share(
            passphrase_bytes,
            encrypted_share,
            &share_nonce_bytes,
            &share_salt,
            recovery_id_raw,
        )?);
    }

    // When threshold==1, shares are raw 32-byte seal keys (vsss-rs cannot split 1-of-N).
    // Any single decrypted share is the key itself.
    let seal_key_bytes: [u8; 32] = if threshold <= 1 {
        let raw = shares
            .into_iter()
            .next()
            .ok_or_else(|| Error::Shamir("No shares available".into()))?;
        raw.try_into()
            .map_err(|_| Error::Shamir("Invalid share length".into()))?
    } else {
        shamir::combine_shares(&shares).map_err(|e| Error::Shamir(e.to_string()))?
    };

    let seal_key = KeyCell::from(seal_key_bytes);

    vault.ask(TryUnseal { seal_key }).await.map_err(|err| {
        error!(?err, "Vault unseal failed");
        Error::VaultError
    })?;

    Ok(())
}

/// §3.3: Generate a fresh seal key, split across current operators, re-encrypt the vault root key.
/// Called after `replace_operator` or `update_shamir_parameters` is approved and all contributors submit.
async fn finalize_rekey(
    db: db::DatabasePool,
    vault: ActorRef<Vault>,
    ordinary_passphrases: HashMap<i32, Vec<u8>>,
    recovery_passphrases: HashMap<i32, Vec<u8>>,
) -> Result<(), Error> {
    let ordinary_count = ordinary_passphrases.len();
    let recovery_count = recovery_passphrases.len();
    let total = ordinary_count + recovery_count;
    let threshold = shamir_threshold(ordinary_count);

    let mut new_seal_key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut new_seal_key_bytes);

    let shares: Vec<Vec<u8>> = if threshold >= 2 {
        shamir::split_key(threshold, total, &new_seal_key_bytes, OsRng)
            .map_err(|e| Error::Shamir(e.to_string()))?
    } else {
        std::iter::repeat_with(|| new_seal_key_bytes.to_vec())
            .take(total)
            .collect()
    };

    let mut conn = db.get().await?;
    let mut shares_iter = shares.into_iter();

    for (operator_id_raw, passphrase_bytes) in ordinary_passphrases {
        let share = shares_iter
            .next()
            .expect("split_key returned enough shares");
        let (encrypted_share, nonce_bytes, share_salt) = encrypt_share(passphrase_bytes, &share)?;

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

    for (recovery_id_raw, passphrase_bytes) in recovery_passphrases {
        let share = shares_iter
            .next()
            .expect("split_key returned enough shares");
        let (encrypted_share, nonce_bytes, share_salt) = encrypt_share(passphrase_bytes, &share)?;

        diesel::replace_into(schema::recovery_operator::table)
            .values((
                schema::recovery_operator::id.eq(recovery_id_raw),
                schema::recovery_operator::share.eq(&encrypted_share),
                schema::recovery_operator::share_nonce.eq(&nonce_bytes),
                schema::recovery_operator::share_salt.eq(&share_salt),
                schema::recovery_operator::created_at.eq(models::SqliteTimestamp::now()),
                schema::recovery_operator::updated_at.eq(models::SqliteTimestamp::now()),
            ))
            .execute(&mut conn)
            .await?;
    }

    drop(conn);

    let new_seal_key = KeyCell::from(new_seal_key_bytes);
    vault
        .ask(RekeyRootKey { new_seal_key })
        .await
        .map_err(|err| {
            error!(?err, "Vault rekey failed");
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
        recovery_count: usize,
    ) -> Result<(), Error> {
        let _ = operator_id; // fixme!: any authenticated operator may announce the committee size. the first call wins
        if !matches!(self.state, CoordinatorState::Idle) {
            return Err(Error::AlreadyBootstrapping);
        }
        if declared_count == 2 && recovery_count == 0 {
            return Err(Error::TwoOperatorsRequireRecovery);
        }
        self.state = CoordinatorState::Bootstrapping {
            declared_count,
            recovery_count,
            passphrases: HashMap::new(),
            recovery_passphrases: HashMap::new(),
        };
        Ok(())
    }

    /// Phase 2 of multi-operator bootstrap: ordinary operator contributes a passphrase.
    /// Returns Ok(true) when all ordinary + recovery operators contributed and bootstrap finalized.
    #[message]
    pub async fn contribute_bootstrap(
        &mut self,
        operator_id: i32,
        mut passphrase: SafeCell<Vec<u8>>,
    ) -> Result<bool, Error> {
        let CoordinatorState::Bootstrapping {
            declared_count,
            recovery_count,
            passphrases,
            recovery_passphrases,
        } = &mut self.state
        else {
            return Err(Error::NotBootstrapping);
        };

        if passphrases.contains_key(&operator_id) {
            return Err(Error::DuplicateContribution);
        }

        let passphrase_bytes = passphrase.read().to_vec();
        passphrases.insert(operator_id, passphrase_bytes);

        if passphrases.len() < *declared_count || recovery_passphrases.len() < *recovery_count {
            return Ok(false);
        }

        let CoordinatorState::Bootstrapping {
            passphrases,
            recovery_passphrases,
            ..
        } = std::mem::replace(&mut self.state, CoordinatorState::Idle)
        else {
            unreachable!()
        };

        finalize_bootstrap(
            self.db.clone(),
            self.vault.clone(),
            passphrases,
            recovery_passphrases,
        )
        .await?;
        Ok(true)
    }

    /// Phase 2 of multi-operator bootstrap: recovery operator contributes a passphrase.
    /// Returns Ok(true) when all contributors are in and bootstrap finalized.
    #[message]
    pub async fn contribute_recovery_bootstrap(
        &mut self,
        recovery_operator_id: i32,
        mut passphrase: SafeCell<Vec<u8>>,
    ) -> Result<bool, Error> {
        let CoordinatorState::Bootstrapping {
            declared_count,
            recovery_count,
            passphrases,
            recovery_passphrases,
        } = &mut self.state
        else {
            return Err(Error::NotBootstrapping);
        };

        if recovery_passphrases.contains_key(&recovery_operator_id) {
            return Err(Error::DuplicateContribution);
        }

        let passphrase_bytes = passphrase.read().to_vec();
        recovery_passphrases.insert(recovery_operator_id, passphrase_bytes);

        if passphrases.len() < *declared_count || recovery_passphrases.len() < *recovery_count {
            return Ok(false);
        }

        let CoordinatorState::Bootstrapping {
            passphrases,
            recovery_passphrases,
            ..
        } = std::mem::replace(&mut self.state, CoordinatorState::Idle)
        else {
            unreachable!()
        };

        finalize_bootstrap(
            self.db.clone(),
            self.vault.clone(),
            passphrases,
            recovery_passphrases,
        )
        .await?;
        Ok(true)
    }

    /// Contribute a passphrase for vault unseal (ordinary operator).
    /// Returns Ok(true) when threshold reached and vault is unsealed.
    #[message]
    pub async fn contribute_unseal(
        &mut self,
        operator_id: i32,
        mut passphrase: SafeCell<Vec<u8>>,
    ) -> Result<bool, Error> {
        self.ensure_unsealing_state().await?;

        let CoordinatorState::Unsealing {
            threshold,
            ordinary_passphrases,
            recovery_passphrases,
        } = &mut self.state
        else {
            return Err(Error::NotUnsealing);
        };

        if ordinary_passphrases.contains_key(&operator_id) {
            return Err(Error::DuplicateContribution);
        }

        let passphrase_bytes = passphrase.read().to_vec();
        ordinary_passphrases.insert(operator_id, passphrase_bytes);

        if ordinary_passphrases.len() + recovery_passphrases.len() < *threshold {
            return Ok(false);
        }

        self.do_finalize_unseal().await
    }

    /// Contribute a passphrase for vault unseal (recovery operator, §3.5).
    /// Recovery operators may contribute during unseal when recovery is active.
    /// Returns Ok(true) when threshold reached and vault is unsealed.
    #[message]
    pub async fn contribute_recovery_unseal(
        &mut self,
        recovery_operator_id: i32,
        mut passphrase: SafeCell<Vec<u8>>,
    ) -> Result<bool, Error> {
        self.ensure_unsealing_state().await?;

        let CoordinatorState::Unsealing {
            threshold,
            ordinary_passphrases,
            recovery_passphrases,
        } = &mut self.state
        else {
            return Err(Error::NotUnsealing);
        };

        if recovery_passphrases.contains_key(&recovery_operator_id) {
            return Err(Error::DuplicateContribution);
        }

        let passphrase_bytes = passphrase.read().to_vec();
        recovery_passphrases.insert(recovery_operator_id, passphrase_bytes);

        if ordinary_passphrases.len() + recovery_passphrases.len() < *threshold {
            return Ok(false);
        }

        self.do_finalize_unseal().await
    }
}

impl VaultCoordinator {
    /// Initializes `CoordinatorState::Unsealing` on first call if still `Idle`.
    /// Threshold is based on ordinary operator count only (§3.4).
    async fn ensure_unsealing_state(&mut self) -> Result<(), Error> {
        if matches!(self.state, CoordinatorState::Idle) {
            let mut conn = self.db.get().await?;
            let ordinary_count: i64 = schema::operator::table
                .count()
                .get_result(&mut conn)
                .await?;
            let threshold = shamir_threshold(usize::try_from(ordinary_count).unwrap_or_default());
            self.state = CoordinatorState::Unsealing {
                threshold,
                ordinary_passphrases: HashMap::new(),
                recovery_passphrases: HashMap::new(),
            };
        }
        Ok(())
    }

    /// Moves state back to Idle and calls finalize_unseal.
    async fn do_finalize_unseal(&mut self) -> Result<bool, Error> {
        let CoordinatorState::Unsealing {
            ordinary_passphrases,
            recovery_passphrases,
            ..
        } = std::mem::replace(&mut self.state, CoordinatorState::Idle)
        else {
            unreachable!()
        };

        finalize_unseal(
            self.db.clone(),
            self.vault.clone(),
            ordinary_passphrases,
            recovery_passphrases,
        )
        .await?;
        Ok(true)
    }

    async fn do_finalize_rekey(&mut self) -> Result<bool, Error> {
        let CoordinatorState::Rekeying {
            passphrases,
            recovery_passphrases,
            ..
        } = std::mem::replace(&mut self.state, CoordinatorState::Idle)
        else {
            unreachable!()
        };

        finalize_rekey(
            self.db.clone(),
            self.vault.clone(),
            passphrases,
            recovery_passphrases,
        )
        .await?;
        Ok(true)
    }
}

#[messages]
impl VaultCoordinator {
    /// Begin Shamir re-key after a key-rotation proposal is approved (§3.3).
    /// Queries the current operator and recovery operator counts from the DB,
    /// then transitions to Rekeying state awaiting contributions from all of them.
    #[message]
    pub async fn start_rekey(&mut self) -> Result<(), Error> {
        if !matches!(self.state, CoordinatorState::Idle) {
            return Err(Error::AlreadyBootstrapping);
        }
        let mut conn = self.db.get().await?;
        let ordinary_count: i64 = schema::operator_identity::table
            .count()
            .get_result(&mut conn)
            .await?;
        let recovery_count: i64 = schema::recovery_operator_identity::table
            .count()
            .get_result(&mut conn)
            .await?;
        self.state = CoordinatorState::Rekeying {
            ordinary_count: ordinary_count as usize,
            recovery_count: recovery_count as usize,
            passphrases: HashMap::new(),
            recovery_passphrases: HashMap::new(),
        };
        Ok(())
    }

    /// Contribute an ordinary operator passphrase for the re-key.
    /// Returns Ok(true) when all contributors have submitted and the re-key is complete.
    #[message]
    pub async fn contribute_rekey(
        &mut self,
        operator_id: i32,
        mut passphrase: SafeCell<Vec<u8>>,
    ) -> Result<bool, Error> {
        let CoordinatorState::Rekeying {
            ordinary_count,
            recovery_count,
            passphrases,
            recovery_passphrases,
        } = &mut self.state
        else {
            return Err(Error::NotRekeying);
        };

        if passphrases.contains_key(&operator_id) {
            return Err(Error::DuplicateContribution);
        }

        passphrases.insert(operator_id, passphrase.read().to_vec());

        if passphrases.len() < *ordinary_count || recovery_passphrases.len() < *recovery_count {
            return Ok(false);
        }

        self.do_finalize_rekey().await
    }

    /// Contribute a recovery operator passphrase for the re-key.
    /// Returns Ok(true) when all contributors have submitted and the re-key is complete.
    #[message]
    pub async fn contribute_recovery_rekey(
        &mut self,
        recovery_operator_id: i32,
        mut passphrase: SafeCell<Vec<u8>>,
    ) -> Result<bool, Error> {
        let CoordinatorState::Rekeying {
            ordinary_count,
            recovery_count,
            passphrases,
            recovery_passphrases,
        } = &mut self.state
        else {
            return Err(Error::NotRekeying);
        };

        if recovery_passphrases.contains_key(&recovery_operator_id) {
            return Err(Error::DuplicateContribution);
        }

        recovery_passphrases.insert(recovery_operator_id, passphrase.read().to_vec());

        if passphrases.len() < *ordinary_count || recovery_passphrases.len() < *recovery_count {
            return Ok(false);
        }

        self.do_finalize_rekey().await
    }
}
