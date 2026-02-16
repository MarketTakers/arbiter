use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use diesel::OptionalExtension as _;
use diesel_async::RunQueryDsl as _;
use ed25519_dalek::VerifyingKey;
use kameo::actor::{ActorRef, Spawn};
use miette::Diagnostic;
use rand::rngs::StdRng;
use smlang::statemachine;
use thiserror::Error;
use tokio::sync::{watch, RwLock};

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

// TODO: Placeholder for secure root key cell implementation
pub struct KeyStorage;

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
    fn move_key(&mut self, _event_data: KeyStorage) -> Result<KeyStorage, ()> {
        todo!()
    }

    #[allow(missing_docs)]
    #[allow(clippy::unused_unit)]
    fn dispose_key(&mut self, _state_data: &KeyStorage) -> Result<(), ()> {
        todo!()
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
}
