use std::sync::Arc;

use diesel::{
    Connection as _, SqliteConnection,
    connection::{SimpleConnection as _, TransactionManager},
};
use diesel_async::{
    AsyncConnection, SimpleAsyncConnection,
    pooled_connection::{AsyncDieselConnectionManager, ManagerConfig, RecyclingMethod},
    sync_connection_wrapper::SyncConnectionWrapper,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use miette::Diagnostic;
use thiserror::Error;

pub mod models;
pub mod schema;

pub type DatabaseConnection = SyncConnectionWrapper<SqliteConnection>;
pub type DatabasePool = diesel_async::pooled_connection::bb8::Pool<DatabaseConnection>;
pub type PoolInitError = diesel_async::pooled_connection::PoolError;
pub type PoolError = diesel_async::pooled_connection::bb8::RunError;

static DB_FILE: &'static str = "arbiter.sqlite";

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Error, Diagnostic, Debug)]
pub enum DatabaseSetupError {
    #[error("Failed to determine home directory")]
    #[diagnostic(code(arbiter::db::home_dir_error))]
    HomeDir(std::io::Error),

    #[error(transparent)]
    #[diagnostic(code(arbiter::db::connection_error))]
    Connection(diesel::ConnectionError),

    #[error(transparent)]
    #[diagnostic(code(arbiter::db::concurrency_error))]
    ConcurrencySetup(diesel::result::Error),

    #[error(transparent)]
    #[diagnostic(code(arbiter::db::migration_error))]
    Migration(Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    #[diagnostic(code(arbiter::db::pool_error))]
    Pool(#[from] PoolInitError),
}

fn database_path() -> Result<std::path::PathBuf, DatabaseSetupError> {
    let arbiter_home = arbiter_proto::home_path().map_err(DatabaseSetupError::HomeDir)?;

    let db_path = arbiter_home.join(DB_FILE);

    Ok(db_path)
}

fn db_config(conn: &mut SqliteConnection) -> Result<(), diesel::result::Error> {
    // fsync only in critical moments
    conn.batch_execute("PRAGMA synchronous = NORMAL;")?;
    // write WAL changes back every 1000 pages, for an in average 1MB WAL file.
    // May affect readers if number is increased
    conn.batch_execute("PRAGMA wal_autocheckpoint = 1000;")?;
    // free some space by truncating possibly massive WAL files from the last run
    conn.batch_execute("PRAGMA wal_checkpoint(TRUNCATE);")?;

    // sqlite foreign keys are disabled by default, enable them for safety
    conn.batch_execute("PRAGMA foreign_keys = ON;")?;

    // better space reclamation
    conn.batch_execute("PRAGMA auto_vacuum = FULL;")?;

    // secure delete, overwrite deleted content with zeros to prevent recovery
    conn.batch_execute("PRAGMA secure_delete = ON;")?;

    Ok(())
}

fn initialize_database(url: &str) -> Result<(), DatabaseSetupError> {
    let mut conn = SqliteConnection::establish(url).map_err(DatabaseSetupError::Connection)?;

    db_config(&mut conn).map_err(DatabaseSetupError::ConcurrencySetup)?;

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(DatabaseSetupError::Migration)?;

    Ok(())
}

pub async fn create_pool() -> Result<DatabasePool, DatabaseSetupError> {
    let database_url = format!(
        "{}?mode=rwc",
        database_path()?
            .to_str()
            .expect("database path is not valid UTF-8")
    );

    initialize_database(&database_url)?;

    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(|url| {
        Box::pin(async move {
            let mut conn = DatabaseConnection::establish(url).await?;

            // see https://fractaledmind.github.io/2023/09/07/enhancing-rails-sqlite-fine-tuning/
            // sleep if the database is busy, this corresponds to up to 9 seconds sleeping time.
            conn.batch_execute("PRAGMA busy_timeout = 9000;")
                .await
                .map_err(diesel::ConnectionError::CouldntSetupConfiguration)?;
            // better write-concurrency
            conn.batch_execute("PRAGMA journal_mode = WAL;")
                .await
                .map_err(diesel::ConnectionError::CouldntSetupConfiguration)?;

            Ok(conn)
        })
    });

    let pool = DatabasePool::builder()
        .build(AsyncDieselConnectionManager::new_with_config(
            database_url,
            config,
        ))
        .await?;

    Ok(pool)
}
