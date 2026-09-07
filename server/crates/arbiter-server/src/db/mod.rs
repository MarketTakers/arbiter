use diesel::{Connection as _, SqliteConnection, connection::SimpleConnection as _};
use diesel_async::{
    AsyncConnection, SimpleAsyncConnection,
    pooled_connection::{AsyncDieselConnectionManager, ManagerConfig},
    sync_connection_wrapper::SyncConnectionWrapper,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use thiserror::Error;
use tracing::info;

pub mod functions;
pub mod models;
pub mod proposal;
pub mod schema;

pub type DatabaseConnection = SyncConnectionWrapper<SqliteConnection>;
pub type DatabasePool = diesel_async::pooled_connection::bb8::Pool<DatabaseConnection>;
pub type PoolInitError = diesel_async::pooled_connection::PoolError;
pub type PoolError = diesel_async::pooled_connection::bb8::RunError;

static DB_FILE: &str = "arbiter.sqlite";

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Error, Debug)]
pub enum DatabaseSetupError {
    #[error(transparent)]
    ConcurrencySetup(diesel::result::Error),

    #[error(transparent)]
    Connection(diesel::ConnectionError),

    #[error("Failed to determine home directory")]
    HomeDir(std::io::Error),

    #[error(transparent)]
    Migration(Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    Pool(#[from] PoolInitError),
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database query error")]
    Connection(#[from] diesel::result::Error),

    #[error("Database connection error")]
    Pool(#[from] PoolError),
}

#[tracing::instrument(level = "info")]
fn database_path() -> Result<std::path::PathBuf, DatabaseSetupError> {
    let arbiter_home = arbiter_proto::home_path().map_err(DatabaseSetupError::HomeDir)?;

    let db_path = arbiter_home.join(DB_FILE);

    Ok(db_path)
}

#[tracing::instrument(level = "info", skip(conn))]
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

#[tracing::instrument(level = "info", skip(url))]
fn initialize_database(url: &str) -> Result<(), DatabaseSetupError> {
    let mut conn = SqliteConnection::establish(url).map_err(DatabaseSetupError::Connection)?;

    db_config(&mut conn).map_err(DatabaseSetupError::ConcurrencySetup)?;

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(DatabaseSetupError::Migration)?;

    info!(%url, "Database initialized successfully");

    Ok(())
}

#[tracing::instrument(level = "info")]
/// Creates a connection pool for the `SQLite` database.
///
/// # Panics
/// Panics if the database path is not valid UTF-8.
pub async fn create_pool(url: Option<&str>) -> Result<DatabasePool, DatabaseSetupError> {
    let database_url = url.map(String::from).unwrap_or(
        database_path()?
            .to_str()
            .expect("database path is not valid UTF-8")
            .to_owned(),
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
            // Per-connection in SQLite: the migration connection enabling it is not enough.
            conn.batch_execute("PRAGMA foreign_keys = ON;")
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

#[mutants::skip]
#[expect(clippy::missing_panics_doc, reason = "Tests oriented function")]
/// Creates a test database pool with a temporary `SQLite` database file.
pub async fn create_test_pool() -> DatabasePool {
    use rand::distr::{Alphanumeric, SampleString as _};

    let tempfile_name = Alphanumeric.sample_string(&mut rand::rng(), 16);

    let file = std::env::temp_dir().join(tempfile_name);
    let url = file
        .to_str()
        .expect("temp file path is not valid UTF-8")
        .to_owned();

    create_pool(Some(&url))
        .await
        .expect("Failed to create test database pool")
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::{
        ExpressionMethods as _,
        dsl::insert_into,
        result::{DatabaseErrorKind, Error as DieselError},
    };
    use diesel_async::RunQueryDsl;

    /// `operator.id` references `operator_identity(id)`. Without `PRAGMA foreign_keys = ON`
    /// on the pooled connection, SQLite accepts a share row for an operator that does not exist.
    #[tokio::test]
    async fn pooled_connections_enforce_foreign_keys() {
        let pool = create_test_pool().await;
        let mut conn = pool.get().await.unwrap();

        let result = insert_into(schema::operator::table)
            .values((
                schema::operator::id.eq(4242),
                schema::operator::share.eq(vec![0u8; 32]),
                schema::operator::share_nonce.eq(vec![0u8; 24]),
                schema::operator::share_salt.eq(vec![0u8; 32]),
            ))
            .execute(&mut conn)
            .await;

        // Specifically a foreign-key violation, not any error: a `NOT NULL` failure or a
        // renamed column would also make `result.is_err()` true without proving the pragma
        // is what rejected the insert.
        assert!(
            matches!(
                result,
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::ForeignKeyViolation,
                    _
                ))
            ),
            "expected a foreign-key violation for a dangling operator_identity reference, got {result:?}"
        );
    }
}
