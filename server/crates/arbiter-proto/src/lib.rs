use crate::proto::auth::AuthChallenge;

pub mod proto {
    tonic::include_proto!("arbiter");

    pub mod auth {
        tonic::include_proto!("arbiter.auth");
    }
}

pub mod transport;

pub static BOOTSTRAP_TOKEN_PATH: &'static str = "bootstrap_token";

pub fn home_path() -> Result<std::path::PathBuf, std::io::Error> {
    static ARBITER_HOME: &'static str = ".arbiter";
    let home_dir = std::env::home_dir().ok_or(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "can not get home directory",
    ))?;

    let arbiter_home = home_dir.join(ARBITER_HOME);
    std::fs::create_dir_all(&arbiter_home)?;

    Ok(arbiter_home)
}

pub fn format_challenge(challenge: &AuthChallenge) -> Vec<u8> {
    let concat_form = format!("{}:{}", challenge.nonce, hex::encode(&challenge.pubkey));
    concat_form.into_bytes().to_vec()
}
