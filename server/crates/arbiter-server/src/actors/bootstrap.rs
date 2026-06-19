use crate::db::{self, DatabasePool, schema};
use arbiter_crypto::safecell::{SafeCell, SafeCellHandle as _};
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

async fn generate_token(path: &Path) -> Result<SafeCell<[u8; TOKEN_LENGTH]>, std::io::Error> {
    let mut cell = SafeCell::new([0u8; TOKEN_LENGTH]);
    {
        let mut buf = cell.write();
        for (slot, b) in buf
            .iter_mut()
            .zip(UnwrapErr(SysRng).sample_iter(Alphanumeric))
        {
            *slot = b;
        }
    }

    let token_str = cell.read_inline(|buf| String::from_utf8_lossy(buf.as_ref()).into_owned());

    write_token_file(path, &token_str).await?;

    Ok(cell)
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
    token: Option<SafeCell<[u8; TOKEN_LENGTH]>>,
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
            (Some(token), Some(path))
        } else {
            (None, None)
        };

        Ok(Self { token, token_path })
    }
}

impl Bootstrapper {
    fn is_correct_token(&mut self, token: &[u8]) -> bool {
        self.token.as_mut().is_some_and(|expected| {
            expected.read_inline(|exp| bool::from(exp.as_ref().ct_eq(token)))
        })
    }
}

#[messages]
impl Bootstrapper {
    #[message]
    pub async fn consume_token(&mut self, token: Vec<u8>) -> bool {
        if self.is_correct_token(&token) {
            self.token = None;
            if let Some(path) = self.token_path.take()
                && let Err(e) = tokio::fs::remove_file(&path).await
            {
                warn!(error = ?e, path = ?path, "Failed to delete bootstrap token file after consumption");
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
    pub fn get_token(&mut self) -> Option<String> {
        self.token
            .as_mut()
            .map(|cell| cell.read_inline(|buf| String::from_utf8_lossy(buf.as_ref()).into_owned()))
    }
}
