pub mod transport;
pub mod url;

use base64::{Engine, prelude::BASE64_STANDARD};
use std::{
    path::PathBuf,
    sync::{LazyLock, RwLock},
};

pub mod proto {
    tonic::include_proto!("arbiter");

    pub mod user_agent {
        tonic::include_proto!("arbiter.user_agent");
    }

    pub mod client {
        tonic::include_proto!("arbiter.client");
    }

    pub mod evm {
        tonic::include_proto!("arbiter.evm");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientMetadata {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
}

pub static BOOTSTRAP_PATH: &str = "bootstrap_token";
static HOME_OVERRIDE: LazyLock<RwLock<Option<PathBuf>>> = LazyLock::new(|| RwLock::new(None));

pub fn set_home_path_override(path: Option<PathBuf>) -> Result<(), std::io::Error> {
    let mut lock = HOME_OVERRIDE
        .write()
        .map_err(|_| std::io::Error::other("home path override lock poisoned"))?;
    *lock = path;
    Ok(())
}

pub fn home_path() -> Result<std::path::PathBuf, std::io::Error> {
    if let Some(path) = HOME_OVERRIDE
        .read()
        .map_err(|_| std::io::Error::other("home path override lock poisoned"))?
        .clone()
    {
        std::fs::create_dir_all(&path)?;
        return Ok(path);
    }

    static ARBITER_HOME: &str = ".arbiter";
    let home_dir = std::env::home_dir().ok_or(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "can not get home directory",
    ))?;

    let arbiter_home = home_dir.join(ARBITER_HOME);
    std::fs::create_dir_all(&arbiter_home)?;

    Ok(arbiter_home)
}

pub fn format_challenge(nonce: i32, pubkey: &[u8]) -> Vec<u8> {
    let concat_form = format!("{}:{}", nonce, BASE64_STANDARD.encode(pubkey));
    concat_form.into_bytes()
}
