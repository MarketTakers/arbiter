mod cli;
mod service;

use clap::Parser;
use cli::{Cli, Command, RunArgs, ServiceCommand};
use rustls::crypto::aws_lc_rs;
use tracing::info;

#[tokio::main]
async fn main() -> miette::Result<()> {
    aws_lc_rs::default_provider().install_default().unwrap();
    init_logging();

    let cli = Cli::parse();

    match cli.command {
        None => run_foreground(RunArgs::default()).await,
        Some(Command::Run(args)) => run_foreground(args).await,
        Some(Command::Service { command }) => match command {
            ServiceCommand::Install(args) => service::install_service(args),
            ServiceCommand::Run(args) => service::run_service_dispatcher(args),
        },
    }
}

async fn run_foreground(args: RunArgs) -> miette::Result<()> {
    info!(addr = %args.listen_addr, "Starting arbiter server");
    arbiter_server::run_server_until_shutdown(
        arbiter_server::RunConfig::new(args.listen_addr, args.data_dir),
        std::future::pending::<()>(),
    )
    .await
}

fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();
}
