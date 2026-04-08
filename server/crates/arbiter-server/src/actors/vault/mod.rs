use chrono::Utc;
use diesel::{
    ExpressionMethods as _, OptionalExtension, QueryDsl, SelectableHelper,
    dsl::{insert_into, update},
};
use diesel_async::{AsyncConnection, RunQueryDsl};
use hmac::Mac as _;
use kameo::{Actor, Reply, actor::ActorRef, messages};
use kameo_actors::message_bus::{MessageBus, Publish};
use strum::{EnumDiscriminants, IntoDiscriminant};
use tracing::{error, info};

use crate::crypto::{
    KeyCell, derive_key,
    encryption::v1::{self, Nonce},
    integrity::v1::HmacSha256,
};
use crate::db::{
    self,
    models::{self, RootKeyHistory},
    schema::{self},
};
use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};

pub mod events {

    #[derive(Clone, Copy)]
    pub struct VaultBootstrapped;

    #[derive(Clone, Copy)]
    pub struct VaultUnsealed;

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
    root_key_history_id: i32,
    root_key: KeyCell,
}

#[derive(Default, EnumDiscriminants)]
#[strum_discriminants(derive(Reply), vis(pub), name(VaultState))]
enum State {
    #[default]
    Unbootstrapped,
    Sealed {
        root_key_history_id: i32,
    },
    Unsealed(Unsealed),
}

/// Manages vault root key and tracks current state of the vault (bootstrapped/unbootstrapped, sealed/unsealed).
/// Provides API for encrypting and decrypting data using the vault root key.
/// Abstraction over database to make sure nonces are never reused and encryption keys are never exposed in plaintext outside of this actor.
#[derive(Actor)]
pub struct Vault {
    db: db::DatabasePool,
    state: State,
    events: ActorRef<MessageBus>,
}

#[messages]
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

    // Exclusive transaction to avoid race condtions if multiple vaults write
    // additional layer of protection against nonce-reuse
    async fn get_new_nonce(pool: &db::DatabasePool, root_key_id: i32) -> Result<Nonce, Error> {
        let mut conn = pool.get().await?;

        let nonce = conn
            .exclusive_transaction(|conn| {
                Box::pin(async move {
                    let current_nonce: Vec<u8> = schema::root_key_history::table
                        .filter(schema::root_key_history::id.eq(root_key_id))
                        .select(schema::root_key_history::data_encryption_nonce)
                        .first(conn)
                        .await?;

                    let mut nonce = Nonce::try_from(current_nonce.as_slice()).map_err(|_| {
                        error!(
                            "Broken database: invalid nonce for root key history id={}",
                            root_key_id
                        );
                        Error::BrokenDatabase
                    })?;
                    nonce.increment();

                    update(schema::root_key_history::table)
                        .filter(schema::root_key_history::id.eq(root_key_id))
                        .set(schema::root_key_history::data_encryption_nonce.eq(nonce.to_vec()))
                        .execute(conn)
                        .await?;

                    Result::<_, Error>::Ok(nonce)
                })
            })
            .await?;

        Ok(nonce)
    }

    fn expect_unsealed(state: &mut State) -> Result<&mut Unsealed, Error> {
        match state {
            State::Unsealed(unsealed) => Ok(unsealed),
            State::Unbootstrapped => Err(Error::NotBootstrapped),
            State::Sealed { .. } => Err(Error::Sealed),
        }
    }

    #[message]
    pub async fn bootstrap(&mut self, seal_key_raw: SafeCell<Vec<u8>>) -> Result<(), Error> {
        if !matches!(self.state, State::Unbootstrapped) {
            return Err(Error::AlreadyBootstrapped);
        }
        let salt = v1::generate_salt();
        let mut seal_key = derive_key(seal_key_raw, &salt);
        let mut root_key = KeyCell::new_secure_random();

        // Zero nonces are fine because they are one-time
        let root_key_nonce = Nonce::default();
        let data_encryption_nonce = Nonce::default();

        let root_key_ciphertext: Vec<u8> = root_key.0.read_inline(|reader| {
            let root_key_reader = reader.as_slice();
            seal_key
                .encrypt(&root_key_nonce, v1::ROOT_KEY_TAG, root_key_reader)
                .map_err(|err| {
                    error!(?err, "Fatal bootstrap error");
                    Error::Encryption(err)
                })
        })?;

        let mut conn = self.db.get().await?;

        let data_encryption_nonce_bytes = data_encryption_nonce.to_vec();
        let root_key_history_id = conn
            .transaction(|conn| {
                Box::pin(async move {
                    let root_key_history_id: i32 = insert_into(schema::root_key_history::table)
                        .values(&models::NewRootKeyHistory {
                            ciphertext: root_key_ciphertext,
                            tag: v1::ROOT_KEY_TAG.to_vec(),
                            root_key_encryption_nonce: root_key_nonce.to_vec(),
                            data_encryption_nonce: data_encryption_nonce_bytes,
                            schema_version: 1,
                            salt: salt.to_vec(),
                        })
                        .returning(schema::root_key_history::id)
                        .get_result(conn)
                        .await?;

                    update(schema::arbiter_settings::table)
                        .set(schema::arbiter_settings::root_key_id.eq(root_key_history_id))
                        .execute(conn)
                        .await?;

                    Result::<_, diesel::result::Error>::Ok(root_key_history_id)
                })
            })
            .await?;

        self.state = State::Unsealed(Unsealed {
            root_key,
            root_key_history_id,
        });

        info!("Vault bootstrapped successfully");
        self.events.tell(Publish(events::VaultBootstrapped)).await;

        Ok(())
    }

    #[message]
    pub async fn try_unseal(&mut self, seal_key_raw: SafeCell<Vec<u8>>) -> Result<(), Error> {
        let State::Sealed {
            root_key_history_id,
        } = &self.state
        else {
            return Err(Error::NotBootstrapped);
        };

        // We don't want to hold connection while doing expensive KDF work
        let current_key = {
            let mut conn = self.db.get().await?;
            schema::root_key_history::table
                .filter(schema::root_key_history::id.eq(*root_key_history_id))
                .select(RootKeyHistory::as_select())
                .first(&mut conn)
                .await?
        };

        let salt = &current_key.salt;
        let salt = v1::Salt::try_from(salt.as_slice()).map_err(|_| {
            error!("Broken database: invalid salt for root key");
            Error::BrokenDatabase
        })?;
        let mut seal_key = derive_key(seal_key_raw, &salt);

        let mut root_key = SafeCell::new(current_key.ciphertext.clone());

        let nonce = v1::Nonce::try_from(current_key.root_key_encryption_nonce.as_slice()).map_err(
            |_| {
                error!("Broken database: invalid nonce for root key");
                Error::BrokenDatabase
            },
        )?;

        seal_key
            .decrypt_in_place(&nonce, v1::ROOT_KEY_TAG, &mut root_key)
            .map_err(|err| {
                error!(?err, "Failed to unseal root key: invalid seal key");
                Error::InvalidKey
            })?;

        self.state = State::Unsealed(Unsealed {
            root_key_history_id: current_key.id,
            root_key: KeyCell::try_from(root_key).map_err(|err| {
                error!(?err, "Broken database: invalid encryption key size");
                Error::BrokenDatabase
            })?,
        });

        info!("Vault unsealed successfully");
        self.events.tell(Publish(events::VaultUnsealed)).await;

        Ok(())
    }

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

        let nonce = v1::Nonce::try_from(row.current_nonce.as_slice()).map_err(|_| {
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
    pub fn sign_integrity(&mut self, mac_input: Vec<u8>) -> Result<(i32, Vec<u8>), Error> {
        let Unsealed {
            root_key,
            root_key_history_id,
        } = Self::expect_unsealed(&mut self.state)?;

        let mut hmac = root_key
            .0
            .read_inline(|k| match HmacSha256::new_from_slice(k) {
                Ok(v) => v,
                Err(_) => unreachable!("HMAC accepts keys of any size"),
            });
        hmac.update(&root_key_history_id.to_be_bytes());
        hmac.update(&mac_input);

        let mac = hmac.finalize().into_bytes().to_vec();
        Ok((*root_key_history_id, mac))
    }

    #[message]
    pub fn verify_integrity(
        &mut self,
        mac_input: Vec<u8>,
        expected_mac: Vec<u8>,
        key_version: i32,
    ) -> Result<bool, Error> {
        let Unsealed {
            root_key,
            root_key_history_id,
        } = Self::expect_unsealed(&mut self.state)?;

        if *root_key_history_id != key_version {
            return Ok(false);
        }

        let mut hmac = root_key
            .0
            .read_inline(|k| match HmacSha256::new_from_slice(k) {
                Ok(v) => v,
                Err(_) => unreachable!("HMAC accepts keys of any size"),
            });
        hmac.update(&key_version.to_be_bytes());
        hmac.update(&mac_input);

        Ok(hmac.verify_slice(&expected_mac).is_ok())
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
        self.events.tell(Publish(events::VaultResealed)).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use diesel::SelectableHelper;

    use diesel_async::RunQueryDsl;

    use crate::{
        actors::GlobalActors,
        db::{self},
    };
    use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};

    use super::*;

    async fn bootstrapped_actor(db: &db::DatabasePool) -> Vault {
        let mut actor = Vault::new(db.clone(), GlobalActors::spawn_message_bus())
            .await
            .unwrap();
        let seal_key = SafeCell::new(b"test-seal-key".to_vec());
        actor.bootstrap(seal_key).await.unwrap();
        actor
    }

    #[tokio::test]
    #[test_log::test]
    async fn nonce_monotonic_even_when_nonce_allocation_interleaves() {
        let db = db::create_test_pool().await;
        let mut actor = bootstrapped_actor(&db).await;
        let root_key_history_id = match actor.state {
            State::Unsealed(Unsealed {
                root_key_history_id,
                ..
            }) => root_key_history_id,
            _ => panic!("expected unsealed state"),
        };

        let n1 = Vault::get_new_nonce(&db, root_key_history_id)
            .await
            .unwrap();
        let n2 = Vault::get_new_nonce(&db, root_key_history_id)
            .await
            .unwrap();
        assert!(n2.to_vec() > n1.to_vec(), "nonce must increase");

        let mut conn = db.get().await.unwrap();
        let root_row: models::RootKeyHistory = schema::root_key_history::table
            .select(models::RootKeyHistory::as_select())
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
