use std::net::SocketAddr;

use arbiter_proto::{proto::arbiter_service_server::ArbiterServiceServer, url::ArbiterUrl};
use arbiter_server::{Server, actors::bootstrap::GetToken, context::ServerContext, db};
use miette::miette;
use tonic::transport::{Identity, ServerTlsConfig};
use tracing::info;

const PORT: u16 = 50051;

#[tokio::main]
async fn main() -> miette::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting arbiter server");

    let db = db::create_pool(None).await?;
    info!("Database ready");

    let context = ServerContext::new(db).await?;

    let addr: SocketAddr = format!("127.0.0.1:{PORT}").parse().expect("valid address");
    info!(%addr, "Starting gRPC server");

    let url = ArbiterUrl {
        host: addr.ip().to_string(),
        port: addr.port(),
        ca_cert: context.tls.ca_cert().clone().into_owned(),
        bootstrap_token: context.actors.bootstrapper.ask(GetToken).await.unwrap(),
    };

    info!(%url, "Server URL");

    let tls = ServerTlsConfig::new().identity(Identity::from_pem(
        context.tls.cert_pem(),
        context.tls.key_pem(),
    ));

    tonic::transport::Server::builder()
        .tls_config(tls)
        .map_err(|err| miette!("Faild to setup TLS: {err}"))?
        .add_service(ArbiterServiceServer::new(Server::new(context)))
        .serve(addr)
        .await
        .map_err(|e| miette::miette!("gRPC server error: {e}"))?;

    unreachable!("gRPC server should run indefinitely");
}
