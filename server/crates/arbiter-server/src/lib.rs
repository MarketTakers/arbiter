#![forbid(unsafe_code)]

use std::{net::SocketAddr, path::PathBuf};

use arbiter_proto::{proto::arbiter_service_server::ArbiterServiceServer, url::ArbiterUrl};
use miette::miette;
use tonic::transport::{Identity, ServerTlsConfig};
use tracing::info;

use crate::{actors::bootstrap::GetToken, context::ServerContext};

pub mod actors;
pub mod context;
pub mod db;
pub mod evm;
pub mod grpc;
pub mod safe_cell;
pub mod utils;

pub struct Server {
    context: ServerContext,
}

impl Server {
    pub fn new(context: ServerContext) -> Self {
        Self { context }
    }
}

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub addr: SocketAddr,
    pub data_dir: Option<PathBuf>,
    pub log_arbiter_url: bool,
}

impl RunConfig {
    pub fn new(addr: SocketAddr, data_dir: Option<PathBuf>) -> Self {
        Self {
            addr,
            data_dir,
            log_arbiter_url: true,
        }
    }
}

pub async fn run_server_until_shutdown<F>(config: RunConfig, shutdown: F) -> miette::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    arbiter_proto::set_home_path_override(config.data_dir.clone())
        .map_err(|err| miette!("failed to set home path override: {err}"))?;

    let db = db::create_pool(None).await?;
    info!(addr = %config.addr, "Database ready");

    let context = ServerContext::new(db).await?;
    info!(addr = %config.addr, "Server context ready");

    if config.log_arbiter_url {
        let url = ArbiterUrl {
            host: config.addr.ip().to_string(),
            port: config.addr.port(),
            ca_cert: context.tls.ca_cert().clone().into_owned(),
            bootstrap_token: context
                .actors
                .bootstrapper
                .ask(GetToken)
                .await
                .map_err(|err| miette!("failed to get bootstrap token from actor: {err}"))?,
        };
        info!(%url, "Server URL");
    }

    let tls = ServerTlsConfig::new().identity(Identity::from_pem(
        context.tls.cert_pem(),
        context.tls.key_pem(),
    ));

    tonic::transport::Server::builder()
        .tls_config(tls)
        .map_err(|err| miette!("Failed to setup TLS: {err}"))?
        .add_service(ArbiterServiceServer::new(Server::new(context)))
        .serve_with_shutdown(config.addr, shutdown)
        .await
        .map_err(|e| miette!("gRPC server error: {e}"))?;

    Ok(())
}
