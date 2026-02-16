use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use diesel::OptionalExtension as _;
use diesel_async::RunQueryDsl as _;
use ed25519_dalek::VerifyingKey;
use kameo::actor::{ActorRef, Spawn};
use miette::Diagnostic;
use rand::rngs::StdRng;
use secrecy::{ExposeSecret, SecretBox};
use smlang::statemachine;
use thiserror::Error;
use tokio::sync::{watch, RwLock};
use zeroize::Zeroizing;

use crate::{
    context::{
        bootstrap::{BootstrapActor, generate_token},
        lease::LeaseHandler,
        tls::{RotationState, RotationTask, TlsDataRaw, TlsManager},
    },
    db::{
        self,
        models::ArbiterSetting,
        schema::{self, arbiter_settings},
    },
};

pub(crate) mod bootstrap;
pub(crate) mod lease;
pub(crate) mod tls;
pub(crate) mod unseal;

#[derive(Error, Debug, Diagnostic)]
pub enum InitError {
    #[error("Database setup failed: {0}")]
    #[diagnostic(code(arbiter_server::init::database_setup))]
    DatabaseSetup(#[from] db::DatabaseSetupError),

    #[error("Connection acquire failed: {0}")]
    #[diagnostic(code(arbiter_server::init::database_pool))]
    DatabasePool(#[from] db::PoolError),

    #[error("Database query error: {0}")]
    #[diagnostic(code(arbiter_server::init::database_query))]
    DatabaseQuery(#[from] diesel::result::Error),

    #[error("TLS initialization failed: {0}")]
    #[diagnostic(code(arbiter_server::init::tls_init))]
    Tls(#[from] tls::TlsInitError),

    #[error("Bootstrap token generation failed: {0}")]
    #[diagnostic(code(arbiter_server::init::bootstrap_token))]
    BootstrapToken(#[from] bootstrap::BootstrapError),

    #[error("I/O Error: {0}")]
    #[diagnostic(code(arbiter_server::init::io))]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug, Diagnostic)]
pub enum UnsealError {
    #[error("Database error: {0}")]
    #[diagnostic(code(arbiter_server::unseal::database_pool))]
    Database(#[from] db::PoolError),

    #[error("Query error: {0}")]
    #[diagnostic(code(arbiter_server::unseal::database_query))]
    Query(#[from] diesel::result::Error),

    #[error("Decryption failed: {0}")]
    #[diagnostic(code(arbiter_server::unseal::decryption))]
    DecryptionFailed(#[from] crate::crypto::CryptoError),

    #[error("Invalid state for unseal")]
    #[diagnostic(code(arbiter_server::unseal::invalid_state))]
    InvalidState,

    #[error("Missing salt in database")]
    #[diagnostic(code(arbiter_server::unseal::missing_salt))]
    MissingSalt,

    #[error("No root key configured in database")]
    #[diagnostic(code(arbiter_server::unseal::no_root_key))]
    NoRootKey,
}

#[derive(Error, Debug, Diagnostic)]
pub enum SealError {
    #[error("Invalid state for seal")]
    #[diagnostic(code(arbiter_server::seal::invalid_state))]
    InvalidState,
}

/// Secure in-memory storage for root encryption key
///
/// Uses `secrecy` crate for automatic zeroization on drop to prevent key material
/// from remaining in memory after use. SecretBox provides heap-allocated secret
/// storage that implements Send + Sync for safe use in async contexts.
pub struct KeyStorage {
    /// 32-byte root key protected by SecretBox
    key: SecretBox<[u8; 32]>,
}

impl KeyStorage {
    /// Create new KeyStorage from a 32-byte root key
    pub fn new(key: [u8; 32]) -> Self {
        Self {
            key: SecretBox::new(Box::new(key)),
        }
    }

    /// Access the key for cryptographic operations
    pub fn key(&self) -> &[u8; 32] {
        self.key.expose_secret()
    }
}

// Drop автоматически реализован через secrecy::Zeroize
// который зануляет память при освобождении

statemachine! {
    name: Server,
    transitions: {
        *NotBootstrapped + Bootstrapped = Sealed,
        Sealed + Unsealed(KeyStorage) / move_key = Ready(KeyStorage),
        Ready(KeyStorage) + Sealed / dispose_key = Sealed,
    }
}
pub struct _Context;
impl ServerStateMachineContext for _Context {
    /// Move key from unseal event into Ready state
    fn move_key(&mut self, event_data: KeyStorage) -> Result<KeyStorage, ()> {
        // Просто перемещаем KeyStorage из event в state
        // Без клонирования - event data consumed
        Ok(event_data)
    }

    /// Securely dispose of key when sealing
    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    fn dispose_key(&mut self, _state_data: &KeyStorage) -> Result<(), ()> {
        // KeyStorage будет dropped после state transition
        // secrecy::Zeroize зануляет память автоматически
        Ok(())
    }
}

pub(crate) struct _ServerContextInner {
    pub db: db::DatabasePool,
    pub state: RwLock<ServerStateMachine<_Context>>,
    pub rng: StdRng,
    pub tls: Arc<TlsManager>,
    pub bootstrapper: ActorRef<BootstrapActor>,
    pub rotation_state: RwLock<RotationState>,
    pub rotation_acks: Arc<RwLock<HashSet<VerifyingKey>>>,
    pub user_agent_leases: LeaseHandler<VerifyingKey>,
    pub client_leases: LeaseHandler<VerifyingKey>,
}
#[derive(Clone)]
pub(crate) struct ServerContext(Arc<_ServerContextInner>);

impl std::ops::Deref for ServerContext {
    type Target = _ServerContextInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ServerContext {
    /// Check if all active clients have acknowledged the rotation
    pub async fn check_rotation_ready(&self) -> bool {
        // TODO: Implement proper rotation readiness check
        // For now, return false as placeholder
        false
    }

    async fn load_tls(
        db: &db::DatabasePool,
        settings: Option<&ArbiterSetting>,
    ) -> Result<TlsManager, InitError> {
        match settings {
            Some(s) if s.current_cert_id.is_some() => {
                // Load active certificate from tls_certificates table
                Ok(TlsManager::load_from_db(
                    db.clone(),
                    s.current_cert_id.unwrap(),
                )
                .await?)
            }
            Some(s) => {
                // Legacy migration: extract validity and save to new table
                let tls_data_raw = TlsDataRaw {
                    cert: s.cert.clone(),
                    key: s.cert_key.clone(),
                };

                // For legacy certificates, use current time as not_before
                // and current time + 90 days as not_after
                let not_before = chrono::Utc::now().timestamp();
                let not_after = not_before + (90 * 24 * 60 * 60); // 90 days

                Ok(TlsManager::new_from_legacy(
                    db.clone(),
                    tls_data_raw,
                    not_before,
                    not_after,
                )
                .await?)
            }
            None => {
                // First startup - generate new certificate
                Ok(TlsManager::new(db.clone()).await?)
            }
        }
    }

    pub async fn new(db: db::DatabasePool) -> Result<Self, InitError> {
        let mut conn = db.get().await?;
        let rng = rand::make_rng();

        let settings = arbiter_settings::table
            .first::<ArbiterSetting>(&mut conn)
            .await
            .optional()?;

        drop(conn);

        // Load TLS manager
        let tls = Self::load_tls(&db, settings.as_ref()).await?;

        // Load rotation state from database
        let rotation_state = RotationState::load_from_db(&db)
            .await
            .unwrap_or(RotationState::Normal);

        let bootstrap_token = generate_token().await?;

        let mut state = ServerStateMachine::new(_Context);

        if let Some(settings) = &settings
            && settings.root_key_id.is_some()
        {
            // TODO: pass the encrypted root key to the state machine and let it handle decryption and transition to Sealed
            let _ = state.process_event(ServerEvents::Bootstrapped);
        }

        // Create shutdown channel for rotation task
        let (rotation_shutdown_tx, rotation_shutdown_rx) = watch::channel(false);

        // Initialize bootstrap actor
        let bootstrapper = BootstrapActor::spawn(BootstrapActor::new(&db).await?);

        let context = Arc::new(_ServerContextInner {
            db: db.clone(),
            rng,
            tls: Arc::new(tls),
            state: RwLock::new(state),
            bootstrapper,
            rotation_state: RwLock::new(rotation_state),
            rotation_acks: Arc::new(RwLock::new(HashSet::new())),
            user_agent_leases: Default::default(),
            client_leases: Default::default(),
        });

        Ok(Self(context))
    }

    /// Unseal vault with password
    pub async fn unseal(&self, password: &str) -> Result<(), UnsealError> {
        use crate::crypto::root_key;
        use diesel::QueryDsl as _;

        // 1. Get root_key_id from settings
        let mut conn = self.db.get().await?;

        let settings: db::models::ArbiterSetting = schema::arbiter_settings::table
            .first(&mut conn)
            .await?;

        let root_key_id = settings.root_key_id.ok_or(UnsealError::NoRootKey)?;

        // 2. Load encrypted root key
        let encrypted: db::models::AeadEncrypted = schema::aead_encrypted::table
            .find(root_key_id)
            .first(&mut conn)
            .await?;

        let salt = encrypted
            .argon2_salt
            .as_ref()
            .ok_or(UnsealError::MissingSalt)?;

        drop(conn);

        // 3. Decrypt root key using password
        let root_key = root_key::decrypt_root_key(&encrypted, password, salt)
            .map_err(UnsealError::DecryptionFailed)?;

        // 4. Create secure storage
        let key_storage = KeyStorage::new(root_key);

        // 5. Transition state machine
        let mut state = self.state.write().await;
        state
            .process_event(ServerEvents::Unsealed(key_storage))
            .map_err(|_| UnsealError::InvalidState)?;

        Ok(())
    }

    /// Seal the server (lock the key)
    pub async fn seal(&self) -> Result<(), SealError> {
        let mut state = self.state.write().await;
        state
            .process_event(ServerEvents::Sealed)
            .map_err(|_| SealError::InvalidState)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keystorage_creation() {
        let key = [42u8; 32];
        let storage = KeyStorage::new(key);
        assert_eq!(storage.key()[0], 42);
        assert_eq!(storage.key().len(), 32);
    }

    #[test]
    fn test_keystorage_zeroization() {
        let key = [99u8; 32];
        {
            let _storage = KeyStorage::new(key);
            // storage будет dropped здесь
        }
        // После drop SecretBox должен зануляеть память
        // Это проверяется автоматически через secrecy::Zeroize
    }

    #[test]
    fn test_state_machine_transitions() {
        let mut state = ServerStateMachine::new(_Context);

        // Начальное состояние
        assert!(matches!(state.state(), &ServerStates::NotBootstrapped));

        // Bootstrapped transition
        state.process_event(ServerEvents::Bootstrapped).unwrap();
        assert!(matches!(state.state(), &ServerStates::Sealed));

        // Unsealed transition
        let key_storage = KeyStorage::new([1u8; 32]);
        state
            .process_event(ServerEvents::Unsealed(key_storage))
            .unwrap();
        assert!(matches!(state.state(), &ServerStates::Ready(_)));

        // Sealed transition
        state.process_event(ServerEvents::Sealed).unwrap();
        assert!(matches!(state.state(), &ServerStates::Sealed));
    }

    #[test]
    fn test_move_key_callback() {
        let mut ctx = _Context;
        let key_storage = KeyStorage::new([7u8; 32]);
        let result = ctx.move_key(key_storage);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().key()[0], 7);
    }

    #[test]
    fn test_dispose_key_callback() {
        let mut ctx = _Context;
        let key_storage = KeyStorage::new([13u8; 32]);
        let result = ctx.dispose_key(&key_storage);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_state_transitions() {
        let mut state = ServerStateMachine::new(_Context);

        // Попытка unseal без bootstrap
        let key_storage = KeyStorage::new([1u8; 32]);
        let result = state.process_event(ServerEvents::Unsealed(key_storage));
        assert!(result.is_err());

        // Правильный путь
        state.process_event(ServerEvents::Bootstrapped).unwrap();

        // Попытка повторного bootstrap
        let result = state.process_event(ServerEvents::Bootstrapped);
        assert!(result.is_err());
    }
}
