use arbiter_proto::{BOOTSTRAP_TOKEN_PATH, home_path};
use diesel::{QueryDsl, dsl::exists, select};
use diesel_async::RunQueryDsl;
use memsafe::MemSafe;
use miette::Diagnostic;
use rand::{RngExt, distr::StandardUniform, make_rng, rngs::StdRng};
use secrecy::SecretString;
use thiserror::Error;
use tracing::info;
use zeroize::{Zeroize, Zeroizing};

use crate::db::{self, schema};

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
