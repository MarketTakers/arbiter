use std::sync::Arc;

use diesel::OptionalExtension as _;
use diesel_async::RunQueryDsl as _;
use kameo::actor::{ActorRef, Spawn};
use miette::Diagnostic;
use thiserror::Error;

use crate::{
    actors::{
        bootstrap::{self, Bootstrapper},
        keyholder::KeyHolder,
    },
    context::tls::{TlsDataRaw, TlsManager},
    db::{self, models::ArbiterSetting, schema::arbiter_settings},
};

pub mod tls;

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

    #[error("KeyHolder initialization failed: {0}")]
    #[diagnostic(code(arbiter_server::init::keyholder_init))]
    KeyHolder(#[from] crate::actors::keyholder::Error),

    #[error("I/O Error: {0}")]
    #[diagnostic(code(arbiter_server::init::io))]
    Io(#[from] std::io::Error),
}

pub struct _ServerContextInner {
    pub db: db::DatabasePool,
    pub tls: TlsManager,
    pub bootstrapper: ActorRef<Bootstrapper>,
    pub keyholder: ActorRef<KeyHolder>,
}
#[derive(Clone)]
pub struct ServerContext(Arc<_ServerContextInner>);

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

        let settings = arbiter_settings::table
            .first::<ArbiterSetting>(&mut conn)
            .await
            .optional()?;

        let tls = Self::load_tls(&mut conn, settings.as_ref()).await?;

        drop(conn);

        Ok(Self(Arc::new(_ServerContextInner {
            bootstrapper: Bootstrapper::spawn(Bootstrapper::new(&db).await?),
            keyholder: KeyHolder::spawn(KeyHolder::new(db.clone()).await?),
            db,
            tls,
        })))
    }
}
