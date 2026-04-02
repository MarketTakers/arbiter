#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::{install_service, run_service_dispatcher};

#[cfg(not(windows))]
pub fn install_service(_: crate::cli::ServiceInstallArgs) -> miette::Result<()> {
    Err(miette::miette!(
        "service install is currently supported only on Windows"
    ))
}

#[cfg(not(windows))]
pub fn run_service_dispatcher(_: crate::cli::ServiceRunArgs) -> miette::Result<()> {
    Err(miette::miette!(
        "service run entrypoint is currently supported only on Windows"
    ))
}
