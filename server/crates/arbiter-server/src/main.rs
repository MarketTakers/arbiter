use arbiter_proto::proto::arbiter_service_server::ArbiterServiceServer;
use arbiter_server::{Server, context::ServerContext, db};
use tracing::info;

#[tokio::main]
async fn main() -> miette::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting arbiter server");

    info!("Initializing database");
    let db = db::create_pool(None).await?;
    info!("Database ready");

    info!("Initializing server context");
    let context = ServerContext::new(db).await?;
    info!("Server context ready");

    let addr = "[::1]:50051".parse().expect("valid address");
    info!(%addr, "Starting gRPC server");

    tonic::transport::Server::builder()
        .add_service(ArbiterServiceServer::new(Server::new(context)))
        .serve(addr)
        .await
        .map_err(|e| miette::miette!("gRPC server error: {e}"))?;

    unreachable!("gRPC server should run indefinitely");
}
