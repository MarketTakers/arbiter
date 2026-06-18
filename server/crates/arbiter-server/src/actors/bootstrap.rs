use crate::db::{self, DatabasePool, schema};
use arbiter_proto::{BOOTSTRAP_PATH, home_path};

use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use kameo::{Actor, messages};
use rand::{RngExt, distr::Alphanumeric, rngs::SysRng};
use rand_core::UnwrapErr;
use std::path::{Path, PathBuf};
use subtle::ConstantTimeEq as _;
use thiserror::Error;
use tracing::warn;
use zeroize::Zeroizing;

const TOKEN_LENGTH: usize = 64;

async fn write_token_file(path: &Path, content: &str) -> Result<(), std::io::Error> {
    tokio::fs::write(path, content.as_bytes()).await?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).await?;
    }

    Ok(())
}

async fn generate_token(path: &Path) -> Result<String, std::io::Error> {
    let token = UnwrapErr(SysRng)
        .sample_iter(Alphanumeric)
        .take(TOKEN_LENGTH)
        .fold(String::default(), |mut accum, char| {
            accum += char.to_string().as_str();
            accum
        });

    write_token_file(path, &token).await?;

    Ok(token)
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] db::PoolError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database query error: {0}")]
    Query(#[from] diesel::result::Error),
}

#[derive(Actor)]
pub struct Bootstrapper {
    token: Option<Zeroizing<String>>,
    token_path: Option<PathBuf>,
}

impl Bootstrapper {
    pub async fn new(db: &DatabasePool) -> Result<Self, Error> {
        let row_count: i64 = {
            let mut conn = db.get().await?;

            schema::operator_client::table
                .count()
                .get_result(&mut conn)
                .await?
        };

        let (token, token_path) = if row_count == 0 {
            let path = home_path()?.join(BOOTSTRAP_PATH);
            let token = generate_token(&path).await?;
            (Some(Zeroizing::new(token)), Some(path))
        } else {
            (None, None)
        };

        Ok(Self { token, token_path })
    }
}

impl Bootstrapper {
    fn is_correct_token(&self, token: &str) -> bool {
        self.token.as_ref().is_some_and(|expected| {
            let expected_bytes = expected.as_bytes();
            let token_bytes = token.as_bytes();

            let choice = expected_bytes.ct_eq(token_bytes);
            bool::from(choice)
        })
    }
}

#[messages]
impl Bootstrapper {
    #[message]
    pub async fn consume_token(&mut self, token: Zeroizing<String>) -> bool {
        if self.is_correct_token(&token) {
            self.token = None;
            if let Some(path) = self.token_path.take() {
                if let Err(e) = tokio::fs::remove_file(&path).await {
                    warn!(error = ?e, path = ?path, "Failed to delete bootstrap token file after consumption");
                }
            }
            true
        } else {
            false
        }
    }
}

#[messages]
impl Bootstrapper {
    #[message]
    pub fn get_token(&self) -> Option<String> {
        self.token.as_ref().map(|token| token.to_string())
    }
}
