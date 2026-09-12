//! Coordinates the multi-operator ceremonies that create and open the vault.
//!
//! The coordinator collects one passphrase per committee member, then hands the
//! assembled material to [`Vault`] in a single message. It owns no Diesel code:
//! everything it reads or writes goes through the [`db::custody`] functions.

use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
use argon2::RECOMMENDED_SALT_LEN;
use kameo::{Actor, actor::ActorRef, error::SendError, messages};
use rand::rngs::SysRng;
use rand_core::{Rng as _, UnwrapErr};

use crate::{
    actors::vault::{self, Bootstrap, TryUnseal, Vault},
    crypto::{KeyCell, derive_key, encryption::v1::Nonce, shamir},
    db::{
        self,
        custody::{CustodyRecord, EncryptedShare},
        models::OperatorId,
    },
};

const SHARE_AAD: &[u8] = b"arbiter/shamir-share/v1";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An ordinary committee is already being coordinated")]
    AlreadyBootstrapping,
    #[error("An unseal is already being coordinated")]
    AlreadyUnsealing,
    #[error("Bootstrap is not in progress")]
    NotBootstrapping,
    #[error("The operator already contributed")]
    DuplicateContribution,
    #[error("The ordinary committee cannot be empty")]
    EmptyCommittee,
    #[error("Two-operator committees are unsupported")]
    UnsupportedCommittee,
    #[error(
        "The ordinary committee cannot exceed {} members",
        shamir::MAX_COMMITTEE_SIZE
    )]
    CommitteeTooLarge,
    #[error("Invalid passphrase")]
    InvalidPassphrase,
    #[error("Broken database")]
    BrokenDatabase,
    #[error("Shamir error: {0}")]
    Shamir(String),
    #[error("Database connection error: {0}")]
    DatabaseConnection(#[from] db::PoolError),
    #[error("Custody storage error: {0}")]
    Custody(#[from] db::custody::Error),
    #[error("Encryption error")]
    Encryption,
    #[error("The vault is already bootstrapped")]
    AlreadyBootstrapped,
    #[error("Vault error")]
    Vault,
}

/// Passphrases gathered so far, in contribution order.
///
/// A `Vec` rather than a map because [`SafeCell`] values are neither cloneable
/// nor hashable, and a committee holds at most
/// [`shamir::MAX_COMMITTEE_SIZE`] of them.
#[derive(Default)]
struct Contributions(Vec<(OperatorId, SafeCell<Vec<u8>>)>);

impl Contributions {
    fn contains(&self, operator_id: OperatorId) -> bool {
        self.0.iter().any(|(id, _)| *id == operator_id)
    }

    fn put(&mut self, operator_id: OperatorId, passphrase: SafeCell<Vec<u8>>) {
        match self.0.iter_mut().find(|(id, _)| *id == operator_id) {
            Some(slot) => slot.1 = passphrase,
            None => self.0.push((operator_id, passphrase)),
        }
    }

    const fn len(&self) -> usize {
        self.0.len()
    }

    fn operators(&self) -> Vec<OperatorId> {
        self.0.iter().map(|(id, _)| *id).collect()
    }
}

enum CoordinatorState {
    Idle,
    Bootstrapping {
        /// The operator that declared the committee. Only they may re-declare
        /// it, which is the way out of a ceremony the others never finish.
        declarer: OperatorId,
        declared_count: usize,
        contributions: Contributions,
        retryable: bool,
    },
    Unsealing {
        threshold: usize,
        contributions: Contributions,
        retryable: bool,
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

/// Explain why a committee size was rejected.
const fn committee_error(declared_count: usize) -> Error {
    match declared_count {
        0 => Error::EmptyCommittee,
        2 => Error::UnsupportedCommittee,
        _ => Error::CommitteeTooLarge,
    }
}

fn encrypt_share(
    passphrase: &mut SafeCell<Vec<u8>>,
    share: &[u8],
) -> Result<EncryptedShare, Error> {
    let mut salt = [0u8; RECOMMENDED_SALT_LEN];
    UnwrapErr(SysRng).fill_bytes(&mut salt);

    let nonce = Nonce::default();
    let ciphertext = derive_key(passphrase, &salt)
        .encrypt(&nonce, SHARE_AAD, share)
        .map_err(|_| Error::Encryption)?;

    Ok(EncryptedShare {
        ciphertext,
        nonce: nonce.to_vec(),
        salt: salt.to_vec(),
    })
}

fn decrypt_share(
    passphrase: &mut SafeCell<Vec<u8>>,
    share: EncryptedShare,
) -> Result<SafeCell<Vec<u8>>, Error> {
    let nonce = Nonce::try_from(share.nonce.as_slice()).map_err(|()| Error::BrokenDatabase)?;

    let mut buffer = SafeCell::new(share.ciphertext);
    derive_key(passphrase, &share.salt)
        .decrypt_in_place(&nonce, SHARE_AAD, &mut buffer)
        .map_err(|_| Error::InvalidPassphrase)?;

    Ok(buffer)
}

const fn bootstrap_error(error: &SendError<Bootstrap, vault::Error>) -> Error {
    match error {
        SendError::HandlerError(vault::Error::AlreadyBootstrapped) => Error::AlreadyBootstrapped,
        _ => Error::Vault,
    }
}

/// Build the custody record and hand it to the vault, which stores it in the
/// same transaction as the root key.
async fn finalize_bootstrap(
    vault: &ActorRef<Vault>,
    contributions: &mut Contributions,
) -> Result<(), Error> {
    let total = contributions.len();
    let threshold = shamir::shamir_threshold(total).ok_or_else(|| committee_error(total))?;

    let mut seal_key = KeyCell::new_secure_random();
    let mut shares = shamir::split_key(threshold, total, &mut seal_key, UnwrapErr(SysRng))
        .map_err(|error| Error::Shamir(error.to_string()))?;

    if shares.len() < total {
        return Err(Error::Shamir("missing share for operator".to_owned()));
    }

    let mut encrypted = Vec::with_capacity(total);
    for ((operator_id, passphrase), share) in contributions.0.iter_mut().zip(shares.iter_mut()) {
        let share = share.read_inline(|share| encrypt_share(passphrase, share))?;
        encrypted.push((*operator_id, share));
    }

    vault
        .ask(Bootstrap {
            seal_key,
            custody: Some(CustodyRecord {
                threshold,
                shares: encrypted,
            }),
        })
        .await
        .map_err(|error| bootstrap_error(&error))
}

/// Reconstruct the seal key from the contributed passphrases and unseal.
async fn finalize_unseal(
    db: &db::DatabasePool,
    vault: &ActorRef<Vault>,
    threshold: usize,
    contributions: &mut Contributions,
) -> Result<(), Error> {
    let stored = {
        let mut conn = db.get().await?;
        db::custody::shares(&mut conn, &contributions.operators()).await?
    };

    let mut plaintext = Vec::with_capacity(stored.len());
    for ((_, passphrase), share) in contributions.0.iter_mut().zip(stored) {
        plaintext.push(decrypt_share(passphrase, share)?);
    }

    let seal_key = shamir::combine_shares(threshold, &mut plaintext)
        .map_err(|error| Error::Shamir(error.to_string()))?;

    vault
        .ask(TryUnseal { seal_key })
        .await
        .map_err(|_| Error::Vault)
}

#[messages]
impl VaultCoordinator {
    /// Announce how many operators will contribute to the bootstrap.
    ///
    /// The declaring operator may re-declare to restart the ceremony; that is
    /// the only way to release a committee whose members never all show up.
    #[message]
    pub fn start_bootstrap(
        &mut self,
        operator_id: OperatorId,
        declared_count: usize,
    ) -> Result<(), Error> {
        if shamir::shamir_threshold(declared_count).is_none() {
            return Err(committee_error(declared_count));
        }

        match &self.state {
            CoordinatorState::Unsealing { .. } => return Err(Error::AlreadyUnsealing),
            CoordinatorState::Bootstrapping { declarer, .. } if *declarer != operator_id => {
                return Err(Error::AlreadyBootstrapping);
            }
            CoordinatorState::Bootstrapping { .. } | CoordinatorState::Idle => {}
        }

        self.state = CoordinatorState::Bootstrapping {
            declarer: operator_id,
            declared_count,
            contributions: Contributions::default(),
            retryable: false,
        };
        Ok(())
    }

    #[message]
    pub async fn contribute_bootstrap(
        &mut self,
        operator_id: OperatorId,
        passphrase: SafeCell<Vec<u8>>,
    ) -> Result<bool, Error> {
        let CoordinatorState::Bootstrapping {
            declared_count,
            contributions,
            retryable,
            ..
        } = &mut self.state
        else {
            return Err(Error::NotBootstrapping);
        };

        if contributions.contains(operator_id) && !*retryable {
            return Err(Error::DuplicateContribution);
        }
        contributions.put(operator_id, passphrase);
        *retryable = false;

        if contributions.len() < *declared_count {
            return Ok(false);
        }

        let state = std::mem::replace(&mut self.state, CoordinatorState::Idle);
        let CoordinatorState::Bootstrapping {
            declarer,
            declared_count,
            mut contributions,
            ..
        } = state
        else {
            unreachable!("state was matched as Bootstrapping above")
        };

        match finalize_bootstrap(&self.vault, &mut contributions).await {
            Ok(()) => Ok(true),
            Err(error) => {
                self.state = CoordinatorState::Bootstrapping {
                    declarer,
                    declared_count,
                    contributions,
                    retryable: true,
                };
                Err(error)
            }
        }
    }

    #[message]
    pub async fn contribute_unseal(
        &mut self,
        operator_id: OperatorId,
        passphrase: SafeCell<Vec<u8>>,
    ) -> Result<bool, Error> {
        if matches!(self.state, CoordinatorState::Idle) {
            let threshold = {
                let mut conn = self.db.get().await?;
                db::custody::threshold(&mut conn).await?
            };
            self.state = CoordinatorState::Unsealing {
                threshold,
                contributions: Contributions::default(),
                retryable: false,
            };
        }

        let CoordinatorState::Unsealing {
            threshold,
            contributions,
            retryable,
        } = &mut self.state
        else {
            return Err(Error::AlreadyBootstrapping);
        };

        if contributions.contains(operator_id) && !*retryable {
            return Err(Error::DuplicateContribution);
        }
        contributions.put(operator_id, passphrase);
        *retryable = false;

        if contributions.len() < *threshold {
            return Ok(false);
        }

        let state = std::mem::replace(&mut self.state, CoordinatorState::Idle);
        let CoordinatorState::Unsealing {
            threshold,
            mut contributions,
            ..
        } = state
        else {
            unreachable!("state was matched as Unsealing above")
        };

        match finalize_unseal(&self.db, &self.vault, threshold, &mut contributions).await {
            Ok(()) => Ok(true),
            Err(error) => {
                self.state = CoordinatorState::Unsealing {
                    threshold,
                    contributions,
                    retryable: true,
                };
                Err(error)
            }
        }
    }
}
