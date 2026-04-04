pub mod transport;
pub mod url;

use base64::{Engine, prelude::BASE64_STANDARD};

pub mod proto {
    tonic::include_proto!("arbiter");

    pub mod shared {
        tonic::include_proto!("arbiter.shared");

        pub mod evm {
            tonic::include_proto!("arbiter.shared.evm");
        }
    }

    pub mod user_agent {
        tonic::include_proto!("arbiter.user_agent");

        pub mod auth {
            tonic::include_proto!("arbiter.user_agent.auth");
        }

        pub mod evm {
            tonic::include_proto!("arbiter.user_agent.evm");
        }

        pub mod sdk_client {
            tonic::include_proto!("arbiter.user_agent.sdk_client");
        }

        pub mod vault {
            tonic::include_proto!("arbiter.user_agent.vault");

            pub mod bootstrap {
                tonic::include_proto!("arbiter.user_agent.vault.bootstrap");
            }

            pub mod unseal {
                tonic::include_proto!("arbiter.user_agent.vault.unseal");
            }
        }
    }

    pub mod client {
        tonic::include_proto!("arbiter.client");

        pub mod auth {
            tonic::include_proto!("arbiter.client.auth");
        }

        pub mod evm {
            tonic::include_proto!("arbiter.client.evm");
        }

        pub mod vault {
            tonic::include_proto!("arbiter.client.vault");
        }
    }

    pub mod evm {
        tonic::include_proto!("arbiter.evm");
    }

    pub mod integrity {
        tonic::include_proto!("arbiter.integrity");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientMetadata {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
}

pub static BOOTSTRAP_PATH: &str = "bootstrap_token";

pub fn home_path() -> Result<std::path::PathBuf, std::io::Error> {
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
