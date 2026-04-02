use std::{net::SocketAddr, path::PathBuf};

use clap::{Args, Parser, Subcommand};

const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:50051";

#[derive(Debug, Parser)]
#[command(name = "arbiter-server")]
#[command(about = "Arbiter gRPC server")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run server in foreground mode.
    Run(RunArgs),
    /// Manage service lifecycle.
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },
}

#[derive(Debug, Clone, Args)]
pub struct RunArgs {
    #[arg(long, default_value = DEFAULT_LISTEN_ADDR)]
    pub listen_addr: SocketAddr,
    #[arg(long)]
    pub data_dir: Option<PathBuf>,
}

impl Default for RunArgs {
    fn default() -> Self {
        Self {
            listen_addr: DEFAULT_LISTEN_ADDR
                .parse()
                .expect("listen address literal must be valid"),
            data_dir: None,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum ServiceCommand {
    /// Install Windows service in Service Control Manager.
    Install(ServiceInstallArgs),
    /// Internal service entrypoint. SCM only.
    #[command(hide = true)]
    Run(ServiceRunArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ServiceInstallArgs {
    #[arg(long)]
    pub start: bool,
    #[arg(long)]
    pub data_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct ServiceRunArgs {
    #[arg(long, default_value = DEFAULT_LISTEN_ADDR)]
    pub listen_addr: SocketAddr,
    #[arg(long)]
    pub data_dir: Option<PathBuf>,
}
