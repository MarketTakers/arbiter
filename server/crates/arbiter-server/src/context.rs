use std::sync::Arc;

use diesel::OptionalExtension as _;
use diesel_async::RunQueryDsl as _;
use ed25519_dalek::VerifyingKey;
use miette::Diagnostic;
use rand::rngs::StdRng;
use smlang::statemachine;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::{
    context::{
        bootstrap::generate_token,
        lease::LeaseHandler,
        tls::{TlsDataRaw, TlsManager},
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

    #[error("I/O Error: {0}")]
    #[diagnostic(code(arbiter_server::init::io))]
    Io(#[from] std::io::Error),
}

// TODO: Placeholder for secure root key cell implementation
pub struct KeyStorage;

statemachine! {
    name: Server,
    transitions: {
        *NotBootstrapped(String) + Bootstrapped = Sealed,
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
    pub tls: TlsManager,
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
    async fn load_tls(
        db: &mut db::DatabaseConnection,
        settings: Option<&ArbiterSetting>,
    ) -> Result<TlsManager, InitError> {
        match &settings {
            Some(settings) => {
                let tls_data_raw = TlsDataRaw {
                    cert: settings.cert.clone(),
                    key: settings.cert_key.clone(),
                };

                Ok(TlsManager::new(Some(tls_data_raw)).await?)
            }
            None => {
                let tls = TlsManager::new(None).await?;
                let tls_data_raw = tls.bytes();

                diesel::insert_into(arbiter_settings::table)
                    .values(&ArbiterSetting {
                        id: 1,
                        root_key_id: None,
                        cert_key: tls_data_raw.key,
                        cert: tls_data_raw.cert,
                    })
                    .execute(db)
                    .await?;

                Ok(tls)
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

        let tls = Self::load_tls(&mut conn, settings.as_ref()).await?;

        drop(conn);

        let bootstrap_token = generate_token().await?;

        let mut state = ServerStateMachine::new(_Context, bootstrap_token);

        if let Some(settings) = &settings
            && settings.root_key_id.is_some()
        {
            // TODO: pass the encrypted root key to the state machine and let it handle decryption and transition to Sealed
            let _ = state.process_event(ServerEvents::Bootstrapped);
        }

        Ok(Self(Arc::new(_ServerContextInner {
            db,
            rng,
            tls,
            state: RwLock::new(state),
            user_agent_leases: Default::default(),
            client_leases: Default::default(),
        })))
    }
}
