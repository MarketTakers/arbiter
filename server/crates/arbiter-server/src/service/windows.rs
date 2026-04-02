use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc,
    time::Duration,
};

use miette::{Context as _, IntoDiagnostic as _, miette};
use windows_service::{
    define_windows_service,
    service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
        ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
};

use crate::cli::{ServiceInstallArgs, ServiceRunArgs};
use arbiter_server::runtime::{RunConfig, run_server_until_shutdown};

const SERVICE_NAME: &str = "ArbiterServer";
const SERVICE_DISPLAY_NAME: &str = "Arbiter Server";

pub fn default_service_data_dir() -> PathBuf {
    let base = std::env::var_os("PROGRAMDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    base.join("Arbiter")
}

pub fn install_service(args: ServiceInstallArgs) -> miette::Result<()> {
    ensure_admin_rights()?;

    let executable = std::env::current_exe().into_diagnostic()?;
    let data_dir = args.data_dir.unwrap_or_else(default_service_data_dir);

    std::fs::create_dir_all(&data_dir)
        .into_diagnostic()
        .with_context(|| format!("failed to create service data dir: {}", data_dir.display()))?;
    ensure_token_acl_contract(&data_dir)?;

    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let manager = ServiceManager::local_computer(None::<&str>, manager_access)
        .into_diagnostic()
        .wrap_err("failed to open Service Control Manager")?;

    let launch_arguments = vec![
        OsString::from("service"),
        OsString::from("run"),
        OsString::from("--data-dir"),
        data_dir.as_os_str().to_os_string(),
    ];

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::OnDemand,
        error_control: ServiceErrorControl::Normal,
        executable_path: executable,
        launch_arguments,
        dependencies: vec![],
        account_name: Some(OsString::from(r"NT AUTHORITY\LocalService")),
        account_password: None,
    };

    let service = manager
        .create_service(
            &service_info,
            ServiceAccess::QUERY_STATUS | ServiceAccess::START,
        )
        .into_diagnostic()
        .wrap_err("failed to create Windows service in SCM")?;

    if args.start {
        service
            .start::<&str>(&[])
            .into_diagnostic()
            .wrap_err("service created but failed to start")?;
    }

    Ok(())
}

pub fn run_service_dispatcher(args: ServiceRunArgs) -> miette::Result<()> {
    SERVICE_RUN_ARGS
        .set(args)
        .map_err(|_| miette!("service runtime args are already initialized"))?;

    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
        .into_diagnostic()
        .wrap_err("failed to start service dispatcher")?;

    Ok(())
}

define_windows_service!(ffi_service_main, service_main);

static SERVICE_RUN_ARGS: std::sync::OnceLock<ServiceRunArgs> = std::sync::OnceLock::new();

fn service_main(_arguments: Vec<OsString>) {
    if let Err(error) = run_service_main() {
        tracing::error!(error = ?error, "Windows service main failed");
    }
}

fn run_service_main() -> miette::Result<()> {
    let args = SERVICE_RUN_ARGS
        .get()
        .cloned()
        .ok_or_else(|| miette!("service run args are missing"))?;

    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

    let status_handle =
        service_control_handler::register(SERVICE_NAME, move |control| match control {
            ServiceControl::Stop => {
                let _ = shutdown_tx.send(());
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        })
        .into_diagnostic()
        .wrap_err("failed to register service control handler")?;

    set_status(
        &status_handle,
        ServiceState::StartPending,
        ServiceControlAccept::empty(),
    )?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .into_diagnostic()
        .wrap_err("failed to build tokio runtime for service")?;

    set_status(
        &status_handle,
        ServiceState::Running,
        ServiceControlAccept::STOP,
    )?;

    let data_dir = args.data_dir.unwrap_or_else(default_service_data_dir);
    let config = RunConfig {
        addr: args.listen_addr,
        data_dir: Some(data_dir),
        log_arbiter_url: true,
    };

    let result = runtime.block_on(run_server_until_shutdown(config, async move {
        let _ = tokio::task::spawn_blocking(move || shutdown_rx.recv()).await;
    }));

    set_status(
        &status_handle,
        ServiceState::Stopped,
        ServiceControlAccept::empty(),
    )?;

    result
}

fn set_status(
    status_handle: &service_control_handler::ServiceStatusHandle,
    current_state: ServiceState,
    controls_accepted: ServiceControlAccept,
) -> miette::Result<()> {
    status_handle
        .set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state,
            controls_accepted,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(10),
            process_id: None,
        })
        .into_diagnostic()
        .wrap_err("failed to update service state")?;

    Ok(())
}

fn ensure_admin_rights() -> miette::Result<()> {
    let status = Command::new("net")
        .arg("session")
        .status()
        .into_diagnostic()
        .wrap_err("failed to check administrator rights")?;

    if status.success() {
        Ok(())
    } else {
        Err(miette!(
            "administrator privileges are required to install Windows service"
        ))
    }
}

fn ensure_token_acl_contract(data_dir: &Path) -> miette::Result<()> {
    // IMPORTANT: This ACL setup is intentionally explicit and should not be simplified away,
    // because service-account and interactive-user access requirements are different in production.
    let target = data_dir.as_os_str();

    let status = Command::new("icacls")
        .arg(target)
        .arg("/grant")
        .arg("*S-1-5-19:(OI)(CI)M")
        .arg("/grant")
        .arg("*S-1-5-32-545:(OI)(CI)RX")
        .arg("/T")
        .arg("/C")
        .status()
        .into_diagnostic()
        .wrap_err("failed to apply ACLs for service data directory")?;

    if status.success() {
        Ok(())
    } else {
        Err(miette!(
            "failed to ensure ACL contract for service data directory"
        ))
    }
}
