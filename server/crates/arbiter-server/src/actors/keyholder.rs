use diesel::{
    ExpressionMethods as _, OptionalExtension, QueryDsl, SelectableHelper,
    dsl::{insert_into, update},
};
use diesel_async::{AsyncConnection, RunQueryDsl};
use kameo::{Actor, Reply, messages};
use memsafe::MemSafe;
use strum::{EnumDiscriminants, IntoDiscriminant};
use tracing::{error, info};

use crate::{
    actors::keyholder::v1::{KeyCell, Nonce},
    db::{
        self,
        models::{self, RootKeyHistory},
        schema::{self},
    },
};

pub mod v1;

#[derive(Default, EnumDiscriminants)]
#[strum_discriminants(derive(Reply), vis(pub))]
enum State {
    #[default]
    Unbootstrapped,
    Sealed {
        root_key_history_id: i32,
    },
    Unsealed {
        root_key_history_id: i32,
        root_key: KeyCell,
    },
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("Keyholder is already bootstrapped")]
    #[diagnostic(code(arbiter::keyholder::already_bootstrapped))]
    AlreadyBootstrapped,
    #[error("Keyholder is not bootstrapped")]
    #[diagnostic(code(arbiter::keyholder::not_bootstrapped))]
    NotBootstrapped,
    #[error("Invalid key provided")]
    #[diagnostic(code(arbiter::keyholder::invalid_key))]
    InvalidKey,

    #[error("Requested aead entry not found")]
    #[diagnostic(code(arbiter::keyholder::aead_not_found))]
    NotFound,

    #[error("Encryption error: {0}")]
    #[diagnostic(code(arbiter::keyholder::encryption_error))]
    Encryption(#[from] chacha20poly1305::aead::Error),

    #[error("Database error: {0}")]
    #[diagnostic(code(arbiter::keyholder::database_error))]
    DatabaseConnection(#[from] db::PoolError),

    #[error("Database transaction error: {0}")]
    #[diagnostic(code(arbiter::keyholder::database_transaction_error))]
    DatabaseTransaction(#[from] diesel::result::Error),

    #[error("Broken database")]
    #[diagnostic(code(arbiter::keyholder::broken_database))]
    BrokenDatabase,
}

/// Manages vault root key and tracks current state of the vault (bootstrapped/unbootstrapped, sealed/unsealed).
/// Provides API for encrypting and decrypting data using the vault root key.
/// Abstraction over database to make sure nonces are never reused and encryption keys are never exposed in plaintext outside of this actor.
#[derive(Actor)]
pub struct KeyHolder {
    db: db::DatabasePool,
    state: State,
}

#[messages]
impl KeyHolder {
    pub async fn new(db: db::DatabasePool) -> Result<Self, Error> {
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

        Ok(Self { db, state })
    }

    // Exclusive transaction to avoid race condtions if multiple keyholders write
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

                    let mut nonce =
                        v1::Nonce::try_from(current_nonce.as_slice()).map_err(|_| {
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

    #[message]
    pub async fn bootstrap(&mut self, seal_key_raw: MemSafe<Vec<u8>>) -> Result<(), Error> {
        if !matches!(self.state, State::Unbootstrapped) {
            return Err(Error::AlreadyBootstrapped);
        }
        let salt = v1::generate_salt();
        let mut seal_key = v1::derive_seal_key(seal_key_raw, &salt);
        let mut root_key = KeyCell::new_secure_random();

        // Zero nonces are fine because they are one-time
        let root_key_nonce = v1::Nonce::default();
        let data_encryption_nonce = v1::Nonce::default();

        let root_key_ciphertext: Vec<u8> = {
            let root_key_reader = root_key.0.read().unwrap();
            let root_key_reader = root_key_reader.as_slice();
            seal_key
                .encrypt(&root_key_nonce, v1::ROOT_KEY_TAG, root_key_reader)
                .map_err(|err| {
                    error!(?err, "Fatal bootstrap error");
                    Error::Encryption(err)
                })?
        };

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

        self.state = State::Unsealed {
            root_key,
            root_key_history_id,
        };

        info!("Keyholder bootstrapped successfully");

        Ok(())
    }

    #[message]
    pub async fn try_unseal(&mut self, seal_key_raw: MemSafe<Vec<u8>>) -> Result<(), Error> {
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
                .select(schema::root_key_history::data_encryption_nonce )
                .select(RootKeyHistory::as_select() )
                .first(&mut conn)
                .await?
        };

        let salt = &current_key.salt;
        let salt = v1::Salt::try_from(salt.as_slice()).map_err(|_| {
            error!("Broken database: invalid salt for root key");
            Error::BrokenDatabase
        })?;
        let mut seal_key = v1::derive_seal_key(seal_key_raw, &salt);

        let mut root_key = MemSafe::new(current_key.ciphertext.clone()).unwrap();

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

        self.state = State::Unsealed {
            root_key_history_id: current_key.id,
            root_key: v1::KeyCell::try_from(root_key).map_err(|err| {
                error!(?err, "Broken database: invalid encryption key size");
                Error::BrokenDatabase
            })?,
        };

        info!("Keyholder unsealed successfully");

        Ok(())
    }

    // Decrypts the `aead_encrypted` entry with the given ID and returns the plaintext
    #[message]
    pub async fn decrypt(&mut self, aead_id: i32) -> Result<MemSafe<Vec<u8>>, Error> {
        let State::Unsealed { root_key, .. } = &mut self.state else {
            return Err(Error::NotBootstrapped);
        };

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
        let mut output = MemSafe::new(row.ciphertext).unwrap();
        root_key.decrypt_in_place(&nonce, v1::TAG, &mut output)?;
        Ok(output)
    }

    // Creates new `aead_encrypted` entry in the database and returns it's ID
    #[message]
    pub async fn create_new(&mut self, mut plaintext: MemSafe<Vec<u8>>) -> Result<i32, Error> {
        let State::Unsealed {
            root_key,
            root_key_history_id,
        } = &mut self.state
        else {
            return Err(Error::NotBootstrapped);
        };

        // Order matters here - `get_new_nonce` acquires connection, so we need to call it before next acquire
        // Borrow checker note: &mut borrow a few lines above is disjoint from this field
        let nonce = Self::get_new_nonce(&self.db, *root_key_history_id).await?;

        let mut ciphertext_buffer = plaintext.write().unwrap();
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
                created_at: chrono::Utc::now().timestamp() as i32,
            })
            .returning(schema::aead_encrypted::id)
            .get_result(&mut conn)
            .await?;

        Ok(aead_id)
    }

    #[message]
    pub fn get_state(&self) -> StateDiscriminants {
        self.state.discriminant()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use diesel::dsl::{insert_into, sql_query, update};
    use diesel_async::RunQueryDsl;
    use kameo::actor::{ActorRef, Spawn as _};
    use memsafe::MemSafe;
    use tokio::task::JoinSet;

    use crate::db::{self, models::ArbiterSetting};

    use super::*;

    async fn seed_settings(pool: &db::DatabasePool) {
        let mut conn = pool.get().await.unwrap();
        insert_into(schema::arbiter_settings::table)
            .values(&ArbiterSetting {
                id: 1,
                root_key_id: None,
                cert_key: vec![],
                cert: vec![],
            })
            .execute(&mut conn)
            .await
            .unwrap();
    }

    async fn bootstrapped_actor(db: &db::DatabasePool) -> KeyHolder {
        seed_settings(db).await;
        let mut actor = KeyHolder::new(db.clone()).await.unwrap();
        let seal_key = MemSafe::new(b"test-seal-key".to_vec()).unwrap();
        actor.bootstrap(seal_key).await.unwrap();
        actor
    }

    async fn write_concurrently(
        actor: ActorRef<KeyHolder>,
        prefix: &'static str,
        count: usize,
    ) -> Vec<(i32, Vec<u8>)> {
        let mut set = JoinSet::new();
        for i in 0..count {
            let actor = actor.clone();
            set.spawn(async move {
                let plaintext = format!("{prefix}-{i}").into_bytes();
                let id = {
                    actor
                        .ask(CreateNew {
                            plaintext: MemSafe::new(plaintext.clone()).unwrap(),
                        })
                        .await
                        .unwrap()
                };
                (id, plaintext)
            });
        }

        let mut out = Vec::with_capacity(count);
        while let Some(res) = set.join_next().await {
            out.push(res.unwrap());
        }
        out
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_bootstrap() {
        let db = db::create_test_pool().await;
        seed_settings(&db).await;
        let mut actor = KeyHolder::new(db.clone()).await.unwrap();

        assert!(matches!(actor.state, State::Unbootstrapped));

        let seal_key = MemSafe::new(b"test-seal-key".to_vec()).unwrap();
        actor.bootstrap(seal_key).await.unwrap();

        assert!(matches!(actor.state, State::Unsealed { .. }));

        let mut conn = db.get().await.unwrap();
        let row: models::RootKeyHistory = schema::root_key_history::table
            .select(models::RootKeyHistory::as_select())
            .first(&mut conn)
            .await
            .unwrap();

        assert_eq!(row.schema_version, 1);
        assert_eq!(row.tag, v1::ROOT_KEY_TAG);
        assert!(!row.ciphertext.is_empty());
        assert!(!row.salt.is_empty());
        assert_eq!(row.data_encryption_nonce, v1::Nonce::default().to_vec());
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_bootstrap_rejects_double() {
        let db = db::create_test_pool().await;
        let mut actor = bootstrapped_actor(&db).await;

        let seal_key2 = MemSafe::new(b"test-seal-key".to_vec()).unwrap();
        let err = actor.bootstrap(seal_key2).await.unwrap_err();
        assert!(matches!(err, Error::AlreadyBootstrapped));
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_create_decrypt_roundtrip() {
        let db = db::create_test_pool().await;
        let mut actor = bootstrapped_actor(&db).await;

        let plaintext = b"hello arbiter";
        let aead_id = actor
            .create_new(MemSafe::new(plaintext.to_vec()).unwrap())
            .await
            .unwrap();

        let mut decrypted = actor.decrypt(aead_id).await.unwrap();
        let decrypted = decrypted.read().unwrap();
        assert_eq!(*decrypted, plaintext);
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_create_new_before_bootstrap_fails() {
        let db = db::create_test_pool().await;
        seed_settings(&db).await;
        let mut actor = KeyHolder::new(db).await.unwrap();

        let err = actor
            .create_new(MemSafe::new(b"data".to_vec()).unwrap())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotBootstrapped));
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_decrypt_before_bootstrap_fails() {
        let db = db::create_test_pool().await;
        seed_settings(&db).await;
        let mut actor = KeyHolder::new(db).await.unwrap();

        let err = actor.decrypt(1).await.unwrap_err();
        assert!(matches!(err, Error::NotBootstrapped));
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_decrypt_nonexistent_returns_not_found() {
        let db = db::create_test_pool().await;
        let mut actor = bootstrapped_actor(&db).await;

        let err = actor.decrypt(9999).await.unwrap_err();
        assert!(matches!(err, Error::NotFound));
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_new_restores_sealed_state() {
        let db = db::create_test_pool().await;
        let actor = bootstrapped_actor(&db).await;
        drop(actor);

        let actor2 = KeyHolder::new(db).await.unwrap();
        assert!(matches!(actor2.state, State::Sealed { .. }));
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_nonce_never_reused() {
        let db = db::create_test_pool().await;
        let mut actor = bootstrapped_actor(&db).await;

        let n = 5;
        let mut ids = Vec::with_capacity(n);
        for i in 0..n {
            let id = actor
                .create_new(MemSafe::new(format!("secret {i}").into_bytes()).unwrap())
                .await
                .unwrap();
            ids.push(id);
        }

        // read all stored nonces from DB
        let mut conn = db.get().await.unwrap();
        let rows: Vec<models::AeadEncrypted> = schema::aead_encrypted::table
            .select(models::AeadEncrypted::as_select())
            .load(&mut conn)
            .await
            .unwrap();

        assert_eq!(rows.len(), n);

        let nonces: Vec<&Vec<u8>> = rows.iter().map(|r| &r.current_nonce).collect();
        let unique: HashSet<&Vec<u8>> = nonces.iter().copied().collect();
        assert_eq!(nonces.len(), unique.len(), "all nonces must be unique");

        // verify nonces are sequential increments from 1
        for (i, row) in rows.iter().enumerate() {
            let mut expected = v1::Nonce::default();
            for _ in 0..=i {
                expected.increment();
            }
            assert_eq!(row.current_nonce, expected.to_vec(), "nonce {i} mismatch");
        }

        // verify data_encryption_nonce on root_key_history tracks the latest nonce
        let root_row: models::RootKeyHistory = schema::root_key_history::table
            .select(models::RootKeyHistory::as_select())
            .first(&mut conn)
            .await
            .unwrap();
        let last_nonce = &rows.last().unwrap().current_nonce;
        assert_eq!(
            &root_row.data_encryption_nonce, last_nonce,
            "root_key_history must track the latest nonce"
        );
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_unseal_correct_password() {
        let db = db::create_test_pool().await;
        let mut actor = bootstrapped_actor(&db).await;

        let plaintext = b"survive a restart";
        let aead_id = actor
            .create_new(MemSafe::new(plaintext.to_vec()).unwrap())
            .await
            .unwrap();
        drop(actor);

        let mut actor = KeyHolder::new(db.clone()).await.unwrap();
        assert!(matches!(actor.state, State::Sealed { .. }));

        let seal_key = MemSafe::new(b"test-seal-key".to_vec()).unwrap();
        actor.try_unseal(seal_key).await.unwrap();
        assert!(matches!(actor.state, State::Unsealed { .. }));

        // previously encrypted data is still decryptable
        let mut decrypted = actor.decrypt(aead_id).await.unwrap();
        assert_eq!(*decrypted.read().unwrap(), plaintext);
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_unseal_wrong_then_correct_password() {
        let db = db::create_test_pool().await;
        let mut actor = bootstrapped_actor(&db).await;

        let plaintext = b"important data";
        let aead_id = actor
            .create_new(MemSafe::new(plaintext.to_vec()).unwrap())
            .await
            .unwrap();
        drop(actor);

        let mut actor = KeyHolder::new(db.clone()).await.unwrap();
        assert!(matches!(actor.state, State::Sealed { .. }));

        // wrong password
        let bad_key = MemSafe::new(b"wrong-password".to_vec()).unwrap();
        let err = actor.try_unseal(bad_key).await.unwrap_err();
        assert!(matches!(err, Error::InvalidKey));
        assert!(
            matches!(actor.state, State::Sealed { .. }),
            "state must remain Sealed after failed attempt"
        );

        // correct password
        let good_key = MemSafe::new(b"test-seal-key".to_vec()).unwrap();
        actor.try_unseal(good_key).await.unwrap();
        assert!(matches!(actor.state, State::Unsealed { .. }));

        let mut decrypted = actor.decrypt(aead_id).await.unwrap();
        assert_eq!(*decrypted.read().unwrap(), plaintext);
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_ciphertext_differs_across_entries() {
        let db = db::create_test_pool().await;
        let mut actor = bootstrapped_actor(&db).await;

        let plaintext = b"same content";
        let id1 = actor
            .create_new(MemSafe::new(plaintext.to_vec()).unwrap())
            .await
            .unwrap();
        let id2 = actor
            .create_new(MemSafe::new(plaintext.to_vec()).unwrap())
            .await
            .unwrap();

        // different nonces => different ciphertext, even for identical plaintext
        let mut conn = db.get().await.unwrap();
        let row1: models::AeadEncrypted = schema::aead_encrypted::table
            .filter(schema::aead_encrypted::id.eq(id1))
            .select(models::AeadEncrypted::as_select())
            .first(&mut conn)
            .await
            .unwrap();
        let row2: models::AeadEncrypted = schema::aead_encrypted::table
            .filter(schema::aead_encrypted::id.eq(id2))
            .select(models::AeadEncrypted::as_select())
            .first(&mut conn)
            .await
            .unwrap();

        assert_ne!(row1.ciphertext, row2.ciphertext);

        // but both decrypt to the same plaintext
        let mut d1 = actor.decrypt(id1).await.unwrap();
        let mut d2 = actor.decrypt(id2).await.unwrap();
        assert_eq!(*d1.read().unwrap(), plaintext);
        assert_eq!(*d2.read().unwrap(), plaintext);
    }

    #[tokio::test]
    #[test_log::test]
    async fn concurrent_create_new_no_duplicate_nonces_() {
        let db = db::create_test_pool().await;
        let actor = KeyHolder::spawn(bootstrapped_actor(&db).await);

        let writes = write_concurrently(actor, "nonce-unique", 32).await;
        assert_eq!(writes.len(), 32);

        let mut conn = db.get().await.unwrap();
        let rows: Vec<models::AeadEncrypted> = schema::aead_encrypted::table
            .select(models::AeadEncrypted::as_select())
            .load(&mut conn)
            .await
            .unwrap();
        assert_eq!(rows.len(), 32);

        let nonces: Vec<&Vec<u8>> = rows.iter().map(|r| &r.current_nonce).collect();
        let unique: HashSet<&Vec<u8>> = nonces.iter().copied().collect();
        assert_eq!(nonces.len(), unique.len(), "all nonces must be unique");
    }

    #[tokio::test]
    #[test_log::test]
    async fn concurrent_create_new_root_nonce_never_moves_backward() {
        let db = db::create_test_pool().await;
        let actor = KeyHolder::spawn(bootstrapped_actor(&db).await);

        write_concurrently(actor, "root-max", 24).await;

        let mut conn = db.get().await.unwrap();
        let rows: Vec<models::AeadEncrypted> = schema::aead_encrypted::table
            .select(models::AeadEncrypted::as_select())
            .load(&mut conn)
            .await
            .unwrap();
        let max_nonce = rows
            .iter()
            .map(|r| r.current_nonce.clone())
            .max()
            .expect("at least one row");

        let root_row: models::RootKeyHistory = schema::root_key_history::table
            .select(models::RootKeyHistory::as_select())
            .first(&mut conn)
            .await
            .unwrap();
        assert_eq!(root_row.data_encryption_nonce, max_nonce);
    }

    #[tokio::test]
    #[test_log::test]
    async fn nonce_monotonic_even_when_nonce_allocation_interleaves() {
        let db = db::create_test_pool().await;
        let mut actor = bootstrapped_actor(&db).await;
        let root_key_history_id = match actor.state {
            State::Unsealed {
                root_key_history_id,
                ..
            } => root_key_history_id,
            _ => panic!("expected unsealed state"),
        };

        let n1 = KeyHolder::get_new_nonce(&db, root_key_history_id)
            .await
            .unwrap();
        let n2 = KeyHolder::get_new_nonce(&db, root_key_history_id)
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
            .create_new(MemSafe::new(b"post-interleave".to_vec()).unwrap())
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

    #[tokio::test]
    #[test_log::test]
    async fn insert_failure_does_not_create_partial_row() {
        let db = db::create_test_pool().await;
        let mut actor = bootstrapped_actor(&db).await;
        let root_key_history_id = match actor.state {
            State::Unsealed {
                root_key_history_id,
                ..
            } => root_key_history_id,
            _ => panic!("expected unsealed state"),
        };

        let mut conn = db.get().await.unwrap();
        let before_count: i64 = schema::aead_encrypted::table
            .count()
            .get_result(&mut conn)
            .await
            .unwrap();
        let before_root_nonce: Vec<u8> = schema::root_key_history::table
            .filter(schema::root_key_history::id.eq(root_key_history_id))
            .select(schema::root_key_history::data_encryption_nonce)
            .first(&mut conn)
            .await
            .unwrap();

        sql_query(
            "CREATE TRIGGER fail_aead_insert BEFORE INSERT ON aead_encrypted BEGIN SELECT RAISE(ABORT, 'forced test failure'); END;",
        )
        .execute(&mut conn)
        .await
        .unwrap();
        drop(conn);

        let err = actor
            .create_new(MemSafe::new(b"should fail".to_vec()).unwrap())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::DatabaseTransaction(_)));

        let mut conn = db.get().await.unwrap();
        sql_query("DROP TRIGGER fail_aead_insert;")
            .execute(&mut conn)
            .await
            .unwrap();

        let after_count: i64 = schema::aead_encrypted::table
            .count()
            .get_result(&mut conn)
            .await
            .unwrap();
        assert_eq!(
            before_count, after_count,
            "failed insert must not create row"
        );

        let after_root_nonce: Vec<u8> = schema::root_key_history::table
            .filter(schema::root_key_history::id.eq(root_key_history_id))
            .select(schema::root_key_history::data_encryption_nonce)
            .first(&mut conn)
            .await
            .unwrap();
        assert!(
            after_root_nonce > before_root_nonce,
            "current behavior allows nonce gap on failed insert"
        );
    }

    #[tokio::test]
    #[test_log::test]
    async fn decrypt_roundtrip_after_high_concurrency() {
        let db = db::create_test_pool().await;
        let actor = KeyHolder::spawn(bootstrapped_actor(&db).await);

        let writes = write_concurrently(actor, "roundtrip", 40).await;
        let expected: HashMap<i32, Vec<u8>> = writes.into_iter().collect();

        let mut decryptor = KeyHolder::new(db.clone()).await.unwrap();
        decryptor
            .try_unseal(MemSafe::new(b"test-seal-key".to_vec()).unwrap())
            .await
            .unwrap();

        for (id, plaintext) in expected {
            let mut decrypted = decryptor.decrypt(id).await.unwrap();
            assert_eq!(*decrypted.read().unwrap(), plaintext);
        }
    }

    // #[tokio::test]
    // #[test_log::test]
    // async fn swapping_ciphertext_and_nonce_between_rows_changes_logical_binding() {
    //     let db = db::create_test_pool().await;
    //     let mut actor = bootstrapped_actor(&db).await;

    //     let plaintext1 = b"entry-one";
    //     let plaintext2 = b"entry-two";
    //     let id1 = actor
    //         .create_new(MemSafe::new(plaintext1.to_vec()).unwrap())
    //         .await
    //         .unwrap();
    //     let id2 = actor
    //         .create_new(MemSafe::new(plaintext2.to_vec()).unwrap())
    //         .await
    //         .unwrap();

    //     let mut conn = db.get().await.unwrap();
    //     let row1: models::AeadEncrypted = schema::aead_encrypted::table
    //         .filter(schema::aead_encrypted::id.eq(id1))
    //         .select(models::AeadEncrypted::as_select())
    //         .first(&mut conn)
    //         .await
    //         .unwrap();
    //     let row2: models::AeadEncrypted = schema::aead_encrypted::table
    //         .filter(schema::aead_encrypted::id.eq(id2))
    //         .select(models::AeadEncrypted::as_select())
    //         .first(&mut conn)
    //         .await
    //         .unwrap();

    //     update(schema::aead_encrypted::table.filter(schema::aead_encrypted::id.eq(id1)))
    //         .set((
    //             schema::aead_encrypted::ciphertext.eq(row2.ciphertext.clone()),
    //             schema::aead_encrypted::current_nonce.eq(row2.current_nonce.clone()),
    //         ))
    //         .execute(&mut conn)
    //         .await
    //         .unwrap();
    //     update(schema::aead_encrypted::table.filter(schema::aead_encrypted::id.eq(id2)))
    //         .set((
    //             schema::aead_encrypted::ciphertext.eq(row1.ciphertext.clone()),
    //             schema::aead_encrypted::current_nonce.eq(row1.current_nonce.clone()),
    //         ))
    //         .execute(&mut conn)
    //         .await
    //         .unwrap();

    //     let mut d1 = actor.decrypt(id1).await.unwrap();
    //     let mut d2 = actor.decrypt(id2).await.unwrap();
    //     assert_eq!(*d1.read().unwrap(), plaintext2);
    //     assert_eq!(*d2.read().unwrap(), plaintext1);
    // }
    #[tokio::test]
    #[test_log::test]
    async fn broken_db_nonce_format_fails_closed() {
        // malformed root_key_history nonce must fail create_new
        let db = db::create_test_pool().await;
        let mut actor = bootstrapped_actor(&db).await;
        let root_key_history_id = match actor.state {
            State::Unsealed {
                root_key_history_id,
                ..
            } => root_key_history_id,
            _ => panic!("expected unsealed state"),
        };

        let mut conn = db.get().await.unwrap();
        update(
            schema::root_key_history::table
                .filter(schema::root_key_history::id.eq(root_key_history_id)),
        )
        .set(schema::root_key_history::data_encryption_nonce.eq(vec![1, 2, 3]))
        .execute(&mut conn)
        .await
        .unwrap();
        drop(conn);

        let err = actor
            .create_new(MemSafe::new(b"must fail".to_vec()).unwrap())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::BrokenDatabase));

        // malformed per-row nonce must fail decrypt
        let db = db::create_test_pool().await;
        let mut actor = bootstrapped_actor(&db).await;
        let id = actor
            .create_new(MemSafe::new(b"decrypt target".to_vec()).unwrap())
            .await
            .unwrap();
        let mut conn = db.get().await.unwrap();
        update(schema::aead_encrypted::table.filter(schema::aead_encrypted::id.eq(id)))
            .set(schema::aead_encrypted::current_nonce.eq(vec![7, 8]))
            .execute(&mut conn)
            .await
            .unwrap();
        drop(conn);

        let err = actor.decrypt(id).await.unwrap_err();
        assert!(matches!(err, Error::BrokenDatabase));
    }
}
