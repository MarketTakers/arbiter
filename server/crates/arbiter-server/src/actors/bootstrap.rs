use crate::db::{self, DatabasePool, schema};
use arbiter_proto::{BOOTSTRAP_PATH, home_path};

use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use kameo::{Actor, messages};
use rand::{RngExt, distr::Alphanumeric, make_rng, rngs::StdRng};
use subtle::ConstantTimeEq as _;
use thiserror::Error;
use zeroize::Zeroizing;

const TOKEN_LENGTH: usize = 64;

pub async fn generate_token() -> Result<String, std::io::Error> {
    let rng: StdRng = make_rng();

    let token = rng.sample_iter(Alphanumeric).take(TOKEN_LENGTH).fold(
        String::default(),
        |mut accum, char| {
            accum += char.to_string().as_str();
            accum
        },
    );

    tokio::fs::write(home_path()?.join(BOOTSTRAP_PATH), token.as_str()).await?;

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

        let token = if row_count == 0 {
            let token = generate_token().await?;
            Some(Zeroizing::new(token))
        } else {
            None
        };

        Ok(Self { token })
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
    pub fn consume_token(&mut self, token: Zeroizing<String>) -> bool {
        if self.is_correct_token(&token) {
            self.token = None;
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
