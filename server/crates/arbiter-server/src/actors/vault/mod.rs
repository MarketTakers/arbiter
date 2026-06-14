use crate::{
    crypto::{
        KeyCell,
        encryption::v1::{self, Nonce},
        integrity::v1::HmacSha256,
    },
    db::{
        self,
        models::{self, RootKeyHistory, RootKeyHistoryId},
        schema,
    },
};
use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};

use chrono::Utc;
use diesel::{
    ExpressionMethods as _, OptionalExtension, QueryDsl, SelectableHelper,
    dsl::{insert_into, update},
};
use diesel_async::{AsyncConnection, RunQueryDsl};
use hmac::{KeyInit as _, Mac as _};
use kameo::{Actor, Reply, actor::ActorRef, messages};
use kameo_actors::message_bus::{MessageBus, Publish};
use strum::{EnumDiscriminants, IntoDiscriminant};
use tracing::{error, info};

pub mod events {
    #[derive(Clone, Copy)]
    pub struct Bootstrapped;

    #[derive(Clone, Copy)]
    pub struct Unsealed;

    #[derive(Clone, Copy)]
    pub struct VaultResealed;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Vault is already bootstrapped")]
    AlreadyBootstrapped,
    #[error("Vault is not bootstrapped")]
    NotBootstrapped,
    #[error("Vault is sealed")]
    Sealed,
    #[error("Invalid key provided")]
    InvalidKey,

    #[error("Requested aead entry not found")]
    NotFound,

    #[error("Encryption error: {0}")]
    Encryption(#[from] chacha20poly1305::aead::Error),

    #[error("Database error: {0}")]
    DatabaseConnection(#[from] db::PoolError),

    #[error("Database transaction error: {0}")]
    DatabaseTransaction(#[from] diesel::result::Error),

    #[error("Broken database")]
    BrokenDatabase,
}

struct Unsealed {
    root_key_history_id: RootKeyHistoryId,
    root_key: KeyCell,
}

#[derive(Default, EnumDiscriminants)]
#[strum_discriminants(derive(Reply), vis(pub), name(VaultState))]
enum State {
    #[default]
    Unbootstrapped,

    Sealed {
        root_key_history_id: RootKeyHistoryId,
    },
    Unsealed(Unsealed),
}

/// Manages vault root key and tracks current state of the vault (bootstrapped/unbootstrapped, sealed/unsealed).
///
/// Provides API for encrypting and decrypting data using the vault root key.
/// Abstraction over database to make sure nonces are never reused and encryption keys are never exposed in plaintext outside of this actor.
#[derive(Actor)]
pub struct Vault {
    db: db::DatabasePool,
    state: State,
    events: ActorRef<MessageBus>,
}

impl Vault {
    pub async fn new(db: db::DatabasePool, events: ActorRef<MessageBus>) -> Result<Self, Error> {
        let state = {
            let mut conn = db.get().await?;

            let (root_key_history,) = schema::arbiter_settings::table
                .left_join(schema::root_key_history::table)
                .select((Option::<RootKeyHistory>::as_select(),))
                .get_result::<(Option<RootKeyHistory>,)>(&mut conn)
                .await?;

            match root_key_history {
                Some(root_key_history) => State::Sealed {
                    root_key_history_id: root_key_history.id,
                },
                None => State::Unbootstrapped,
            }
        };

        Ok(Self { db, state, events })
    }

    // Exclusive transaction to avoid race conditions if multiple vaults write
    // additional layer of protection against nonce-reuse
    async fn get_new_nonce(
        pool: &db::DatabasePool,
        root_key_id: RootKeyHistoryId,
    ) -> Result<Nonce, Error> {
        let mut conn = pool.get().await?;

        let nonce = conn
            .exclusive_transaction(async |conn| {
                let current_nonce: Vec<u8> = schema::root_key_history::table
                    .filter(schema::root_key_history::id.eq(root_key_id))
                    .select(schema::root_key_history::data_encryption_nonce)
                    .first(&mut *conn)
                    .await?;

                let mut nonce = Nonce::try_from(current_nonce.as_slice()).map_err(|()| {
                    error!(
                        "Broken database: invalid nonce for root key history id={:#?}",
                        root_key_id
                    );
                    Error::BrokenDatabase
                })?;
                nonce.increment();

                update(schema::root_key_history::table)
                    .filter(schema::root_key_history::id.eq(root_key_id))
                    .set(schema::root_key_history::data_encryption_nonce.eq(nonce.to_vec()))
                    .execute(&mut *conn)
                    .await?;

                Result::<_, Error>::Ok(nonce)
            })
            .await?;

        Ok(nonce)
    }

    const fn expect_unsealed(state: &mut State) -> Result<&mut Unsealed, Error> {
        match state {
            State::Unsealed(unsealed) => Ok(unsealed),
            State::Unbootstrapped => Err(Error::NotBootstrapped),
            State::Sealed { .. } => Err(Error::Sealed),
        }
    }
}

#[messages]
impl Vault {
    #[message]
    pub async fn bootstrap(&mut self, mut seal_key: KeyCell) -> Result<(), Error> {
        if !matches!(&self.state, State::Unbootstrapped) {
            return Err(Error::AlreadyBootstrapped);
        }

        let mut root_key = KeyCell::new_secure_random();

        // Zero nonces are fine because they are one-time
        let root_key_nonce = Nonce::default();
        let data_encryption_nonce = Nonce::default();

        // Generate salt (kept for schema compat)
        let root_key_salt = v1::generate_salt();

        let root_key_ciphertext: Vec<u8> = root_key.0.read_inline(|rk| {
            seal_key
                .encrypt(&root_key_nonce, v1::ROOT_KEY_TAG, rk.as_slice())
                .map_err(|err| {
                    error!(?err, "Fatal bootstrap error");
                    Error::Encryption(err)
                })
        })?;

        let data_encryption_nonce_bytes = data_encryption_nonce.to_vec();
        let mut conn = self.db.get().await?;

        let root_key_history_id = conn
            .transaction(async |conn| {
                let root_key_history_id = insert_into(schema::root_key_history::table)
                    .values(&models::NewRootKeyHistory {
                        ciphertext: root_key_ciphertext.clone(),
                        tag: v1::ROOT_KEY_TAG.to_vec(),
                        root_key_encryption_nonce: root_key_nonce.to_vec(),
                        data_encryption_nonce: data_encryption_nonce_bytes.clone(),
                        schema_version: 1,
                        salt: root_key_salt.to_vec(),
                    })
                    .returning(schema::root_key_history::id)
                    .get_result(&mut *conn)
                    .await?;

                update(schema::arbiter_settings::table)
                    .set(schema::arbiter_settings::root_key_id.eq(root_key_history_id))
                    .execute(&mut *conn)
                    .await?;

                Result::<_, diesel::result::Error>::Ok(RootKeyHistoryId::from_raw(
                    root_key_history_id,
                ))
            })
            .await?;

        self.state = State::Unsealed(Unsealed {
            root_key,
            root_key_history_id,
        });

        info!("Vault bootstrapped successfully");
        let _ = self.events.tell(Publish(events::Bootstrapped)).await;

        Ok(())
    }

    #[message]
    pub async fn try_unseal(&mut self, mut seal_key: KeyCell) -> Result<(), Error> {
        let State::Sealed {
            root_key_history_id,
        } = &self.state
        else {
            return Err(Error::NotBootstrapped);
        };
        let root_key_history_id = *root_key_history_id;

        // We don't want to hold connection while doing expensive work
        let current_key = {
            let mut conn = self.db.get().await?;
            schema::root_key_history::table
                .filter(schema::root_key_history::id.eq(root_key_history_id))
                .select(RootKeyHistory::as_select())
                .first(&mut conn)
                .await?
        };

        let nonce =
            Nonce::try_from(current_key.root_key_encryption_nonce.as_slice()).map_err(|()| {
                error!("Broken database: invalid nonce for root key");
                Error::BrokenDatabase
            })?;

        let mut root_key_bytes = SafeCell::new(current_key.ciphertext.clone());
        seal_key
            .decrypt_in_place(&nonce, v1::ROOT_KEY_TAG, &mut root_key_bytes)
            .map_err(|err| {
                error!(?err, "Failed to unseal root key: invalid seal key");
                Error::InvalidKey
            })?;

        let root_key = KeyCell::try_from(root_key_bytes).map_err(|()| {
            error!("Broken database: invalid encryption key size");
            Error::BrokenDatabase
        })?;

        self.state = State::Unsealed(Unsealed {
            root_key_history_id: current_key.id,
            root_key,
        });

        info!("Vault unsealed successfully");
        let _ = self.events.tell(Publish(events::Unsealed)).await;

        Ok(())
    }

    /// Re-encrypts the root key with `new_seal_key` and records a new root_key_history row.
    /// Called after a Shamir re-key so the old seal key is no longer sufficient to unseal.
    #[message]
    pub async fn rekey_root_key(&mut self, mut new_seal_key: KeyCell) -> Result<(), Error> {
        let Unsealed {
            root_key,
            root_key_history_id,
        } = Self::expect_unsealed(&mut self.state)?;

        let new_nonce = Nonce::default();
        let new_salt = v1::generate_salt();

        let new_ciphertext: Vec<u8> = root_key.0.read_inline(|rk| {
            new_seal_key
                .encrypt(&new_nonce, v1::ROOT_KEY_TAG, rk.as_slice())
                .map_err(|err| {
                    error!(?err, "Fatal rekey error");
                    Error::Encryption(err)
                })
        })?;

        let data_encryption_nonce = Nonce::default();

        let mut conn = self.db.get().await?;
        let new_root_key_history_id: i32 = conn
            .transaction(async |conn| {
                let new_id = insert_into(schema::root_key_history::table)
                    .values(&models::NewRootKeyHistory {
                        ciphertext: new_ciphertext,
                        tag: v1::ROOT_KEY_TAG.to_vec(),
                        root_key_encryption_nonce: new_nonce.to_vec(),
                        data_encryption_nonce: data_encryption_nonce.to_vec(),
                        schema_version: 1,
                        salt: new_salt.to_vec(),
                    })
                    .returning(schema::root_key_history::id)
                    .get_result::<i32>(&mut *conn)
                    .await?;

                update(schema::arbiter_settings::table)
                    .set(schema::arbiter_settings::root_key_id.eq(new_id))
                    .execute(&mut *conn)
                    .await?;

                Result::<_, diesel::result::Error>::Ok(new_id)
            })
            .await?;

        *root_key_history_id = RootKeyHistoryId::from_raw(new_root_key_history_id);
        info!("Vault root key rekeyed successfully");
        Ok(())
    }

    #[message]
    pub async fn seal(&mut self) -> Result<(), Error> {
        let Unsealed {
            root_key_history_id,
            ..
        } = Self::expect_unsealed(&mut self.state)?;

        self.state = State::Sealed {
            root_key_history_id: *root_key_history_id,
        };
        let _ = self.events.tell(Publish(events::VaultResealed)).await;
        Ok(())
    }
}

// Server-side cryptographic operations
#[messages]
impl Vault {
    #[message]
    pub async fn decrypt(&mut self, aead_id: i32) -> Result<SafeCell<Vec<u8>>, Error> {
        let Unsealed { root_key, .. } = Self::expect_unsealed(&mut self.state)?;

        let row: models::AeadEncrypted = {
            let mut conn = self.db.get().await?;
            schema::aead_encrypted::table
                .select(models::AeadEncrypted::as_select())
                .filter(schema::aead_encrypted::id.eq(aead_id))
                .first(&mut conn)
                .await
                .optional()?
                .ok_or(Error::NotFound)?
        };

        let nonce = Nonce::try_from(row.current_nonce.as_slice()).map_err(|()| {
            error!(
                "Broken database: invalid nonce for aead_encrypted id={}",
                aead_id
            );
            Error::BrokenDatabase
        })?;
        let mut output = SafeCell::new(row.ciphertext);
        root_key.decrypt_in_place(&nonce, v1::TAG, &mut output)?;
        Ok(output)
    }

    // Creates new `aead_encrypted` entry in the database and returns it's ID
    #[message]
    pub async fn create_new(&mut self, mut plaintext: SafeCell<Vec<u8>>) -> Result<i32, Error> {
        let Unsealed {
            root_key,
            root_key_history_id,
        } = Self::expect_unsealed(&mut self.state)?;

        // Order matters here - `get_new_nonce` acquires connection, so we need to call it before next acquire
        // Borrow checker note: &mut borrow a few lines above is disjoint from this field
        let nonce = Self::get_new_nonce(&self.db, *root_key_history_id).await?;

        let mut ciphertext_buffer = plaintext.write();
        let ciphertext_buffer: &mut Vec<u8> = ciphertext_buffer.as_mut();
        root_key.encrypt_in_place(&nonce, v1::TAG, &mut *ciphertext_buffer)?;

        let ciphertext = std::mem::take(ciphertext_buffer);

        let mut conn = self.db.get().await?;
        let aead_id: i32 = insert_into(schema::aead_encrypted::table)
            .values(&models::NewAeadEncrypted {
                ciphertext,
                tag: v1::TAG.to_vec(),
                current_nonce: nonce.to_vec(),
                schema_version: 1,
                associated_root_key_id: *root_key_history_id,
                created_at: Utc::now().into(),
            })
            .returning(schema::aead_encrypted::id)
            .get_result(&mut conn)
            .await?;

        Ok(aead_id)
    }

    #[message]
    pub fn get_state(&self) -> VaultState {
        self.state.discriminant()
    }

    #[message]
    pub fn sign_integrity(
        &mut self,
        mac_input: Vec<u8>,
    ) -> Result<(RootKeyHistoryId, Vec<u8>), Error> {
        let Unsealed {
            root_key,
            root_key_history_id,
        } = Self::expect_unsealed(&mut self.state)?;

        let mut hmac = root_key.0.read_inline(|k| {
            HmacSha256::new_from_slice(k)
                .unwrap_or_else(|_| unreachable!("HMAC accepts keys of any size"))
        });
        hmac.update(&root_key_history_id.to_raw().to_be_bytes());
        hmac.update(&mac_input);

        let mac = hmac.finalize().into_bytes().to_vec();
        Ok((*root_key_history_id, mac))
    }

    #[message]
    pub fn verify_integrity(
        &mut self,
        mac_input: Vec<u8>,
        expected_mac: Vec<u8>,
        key_version: RootKeyHistoryId,
    ) -> Result<bool, Error> {
        let Unsealed {
            root_key,
            root_key_history_id,
        } = Self::expect_unsealed(&mut self.state)?;

        if *root_key_history_id != key_version {
            return Ok(false);
        }

        let mut hmac = root_key.0.read_inline(|k| {
            HmacSha256::new_from_slice(k)
                .unwrap_or_else(|_| unreachable!("HMAC accepts keys of any size"))
        });
        hmac.update(&key_version.to_raw().to_be_bytes());
        hmac.update(&mac_input);

        Ok(hmac.verify_slice(&expected_mac).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use crate::actors::GlobalActors;
    use arbiter_crypto::safecell::SafeCellHandle as _;

    use super::*;

    async fn bootstrapped_actor(db: &db::DatabasePool) -> Vault {
        let mut actor = Vault::new(db.clone(), GlobalActors::spawn_message_bus())
            .await
            .unwrap();
        actor.bootstrap(KeyCell::from([0u8; 32])).await.unwrap();
        actor
    }

    #[tokio::test]
    #[test_log::test]
    async fn nonce_monotonic_even_when_nonce_allocation_interleaves() {
        let db = db::create_test_pool().await;
        let mut actor = bootstrapped_actor(&db).await;
        let State::Unsealed(Unsealed {
            root_key_history_id,
            ..
        }) = actor.state
        else {
            panic!("expected unsealed state");
        };

        let n1 = Vault::get_new_nonce(&db, root_key_history_id)
            .await
            .unwrap();
        let n2 = Vault::get_new_nonce(&db, root_key_history_id)
            .await
            .unwrap();
        assert!(n2.to_vec() > n1.to_vec(), "nonce must increase");

        let mut conn = db.get().await.unwrap();
        let root_row: RootKeyHistory = schema::root_key_history::table
            .select(RootKeyHistory::as_select())
            .first(&mut conn)
            .await
            .unwrap();
        assert_eq!(root_row.data_encryption_nonce, n2.to_vec());

        let id = actor
            .create_new(SafeCell::new(b"post-interleave".to_vec()))
            .await
            .unwrap();
        let row: models::AeadEncrypted = schema::aead_encrypted::table
            .filter(schema::aead_encrypted::id.eq(id))
            .select(models::AeadEncrypted::as_select())
            .first(&mut conn)
            .await
            .unwrap();
        assert!(
            row.current_nonce > n2.to_vec(),
            "next write must advance nonce"
        );
    }
}
