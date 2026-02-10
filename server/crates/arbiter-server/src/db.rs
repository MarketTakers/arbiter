use diesel::{Connection as _, SqliteConnection, connection::SimpleConnection as _};
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use miette::Diagnostic;
use thiserror::Error;

pub mod models;
pub mod schema;

pub type Database = SyncConnectionWrapper<SqliteConnection>;

static ARBITER_HOME: &'static str = ".arbiter";
static DB_FILE: &'static str = "db.sqlite";

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Error, Diagnostic, Debug)]
pub enum DatabaseSetupError {
    #[error("Failed to determine home directory")]
    #[diagnostic(code(arbiter::db::home_dir_error))]
    HomeDir(Option<std::io::Error>),

    #[error(transparent)]
    #[diagnostic(code(arbiter::db::connection_error))]
    Connection(diesel::ConnectionError),

    #[error(transparent)]
    #[diagnostic(code(arbiter::db::concurrency_error))]
    ConcurrencySetup(diesel::result::Error),

    #[error(transparent)]
    #[diagnostic(code(arbiter::db::migration_error))]
    Migration(Box<dyn std::error::Error + Send + Sync>),
}

fn database_path() -> Result<std::path::PathBuf, DatabaseSetupError> {
    let home_dir = std::env::home_dir().ok_or_else(|| DatabaseSetupError::HomeDir(None))?;

    let arbiter_home = home_dir.join(ARBITER_HOME);

    let db_path = arbiter_home.join(DB_FILE);

    std::fs::create_dir_all(arbiter_home)
        .map_err(|err| DatabaseSetupError::HomeDir(Some(err)))?;

    Ok(db_path)
}

fn setup_concurrency(conn: &mut SqliteConnection) -> Result<(), diesel::result::Error> {
    // see https://fractaledmind.github.io/2023/09/07/enhancing-rails-sqlite-fine-tuning/
    // sleep if the database is busy, this corresponds to up to 2 seconds sleeping time.
    conn.batch_execute("PRAGMA busy_timeout = 2000;")?;
    // better write-concurrency
    conn.batch_execute("PRAGMA journal_mode = WAL;")?;
    // fsync only in critical moments
    conn.batch_execute("PRAGMA synchronous = NORMAL;")?;
    // write WAL changes back every 1000 pages, for an in average 1MB WAL file.
    // May affect readers if number is increased
    conn.batch_execute("PRAGMA wal_autocheckpoint = 1000;")?;
    // free some space by truncating possibly massive WAL files from the last run
    conn.batch_execute("PRAGMA wal_checkpoint(TRUNCATE);")?;

    Ok(())
}

#[tracing::instrument]
pub fn connect() -> Result<Database, DatabaseSetupError> {
    let database_url = format!(
        "{}?mode=rwc",
        database_path()?
            .to_str()
            .ok_or_else(|| DatabaseSetupError::HomeDir(None))?
    );
    let mut conn =
        SqliteConnection::establish(&database_url).map_err(DatabaseSetupError::Connection)?;

    setup_concurrency(&mut conn).map_err(DatabaseSetupError::ConcurrencySetup)?;

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(DatabaseSetupError::Migration)?;

    Ok(SyncConnectionWrapper::new(conn))
}
