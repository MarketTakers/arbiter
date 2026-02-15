use arbiter_proto::{BOOTSTRAP_TOKEN_PATH, home_path};
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use kameo::{Actor, messages};
use miette::Diagnostic;
use rand::{RngExt, distr::StandardUniform, make_rng, rngs::StdRng};
use thiserror::Error;
use tracing::info;

use crate::db::{self, DatabasePool, schema};

const TOKEN_LENGTH: usize = 64;

pub async fn generate_token() -> Result<String, std::io::Error> {
    let rng: StdRng = make_rng();

    let token: String = rng
        .sample_iter::<char, _>(StandardUniform)
        .take(TOKEN_LENGTH)
        .fold(Default::default(), |mut accum, char| {
            accum += char.to_string().as_str();
            accum
        });

    tokio::fs::write(home_path()?.join(BOOTSTRAP_TOKEN_PATH), token.as_str()).await?;

    Ok(token)
}

#[derive(Error, Debug, Diagnostic)]
pub enum BootstrapError {
    #[error("Database error: {0}")]
    #[diagnostic(code(arbiter_server::bootstrap::database))]
    Database(#[from] db::PoolError),

    #[error("Database query error: {0}")]
    #[diagnostic(code(arbiter_server::bootstrap::database_query))]
    Query(#[from] diesel::result::Error),

    #[error("I/O error: {0}")]
    #[diagnostic(code(arbiter_server::bootstrap::io))]
    Io(#[from] std::io::Error),
}

#[derive(Actor)]
pub struct BootstrapActor {
    token: Option<String>,
}

impl BootstrapActor {
    pub async fn new(db: &DatabasePool) -> Result<Self, BootstrapError> {
        let mut conn = db.get().await?;

        let row_count: i64 = schema::useragent_client::table
            .count()
            .get_result(&mut conn)
            .await?;

        drop(conn);

        let token = if row_count == 0 {
            let token = generate_token().await?;
            info!(%token, "Generated bootstrap token");
            tokio::fs::write(home_path()?.join(BOOTSTRAP_TOKEN_PATH), token.as_str()).await?;
            Some(token)
        } else {
            None
        };

        Ok(Self { token })
    }

    #[cfg(test)]
    pub fn get_token(&self) -> Option<String> {
        self.token.clone()
    }
}

#[messages]
impl BootstrapActor {
    #[message]
    pub fn is_correct_token(&self, token: String) -> bool {
        match &self.token {
            Some(expected) => *expected == token,
            None => false,
        }
    }

    #[message]
    pub fn consume_token(&mut self, token: String) -> bool {
        if self.is_correct_token(token) {
            self.token = None;
            true
        } else {
            false
        }
    }
}
