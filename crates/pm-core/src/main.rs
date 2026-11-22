//! The Rust Portmaster core binary.
//!
//! Starts the database system, the configuration system and the HTTP API, and
//! keeps running until it receives a shutdown signal. The restart exit code is
//! [`pm_core::cli::RESTART_EXIT_CODE`], like in the Go core.

use std::process::ExitCode;
use std::sync::Arc;

use pm_core::cli;
use pm_core::service::Service;

#[cfg(windows)]
const SERVICE_NAME: &str = "PortmasterCore";

#[cfg(windows)]
const ERROR_FAILED_SERVICE_CONTROLLER_CONNECT: i32 = 1063;

#[cfg(windows)]
windows_service::define_windows_service!(ffi_service_main, win_service_main);

fn main() -> ExitCode {
    let options = match cli::parse(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            eprintln!();
            eprintln!("{}", cli::help());
            return ExitCode::FAILURE;
        }
    };

    if options.print_version {
        println!("{}", cli::version());
        return ExitCode::SUCCESS;
    }
    if options.help {
        println!("{}", cli::help());
        return ExitCode::SUCCESS;
    }
    if options.recover_iptables {
        #[cfg(target_os = "linux")]
        {
            if !nix::unistd::geteuid().is_root() {
                eprintln!("Error: recovering iptables requires root privileges. Please run with sudo or as root.");
                return ExitCode::FAILURE;
            }
            use pm_interception::iptables::{self, Cmd};
            let mut failed = false;
            if let Err(error) = iptables::deactivate(&Cmd::v4(), iptables::ONCE, iptables::V4_CHAINS) {
                eprintln!("failed to clean up IPv4 iptables: {error}");
                failed = true;
            }
            if let Err(error) = iptables::deactivate(&Cmd::v6(), iptables::ONCE, iptables::V6_CHAINS) {
                eprintln!("failed to clean up IPv6 iptables: {error}");
                failed = true;
            }
            if failed {
                return ExitCode::FAILURE;
            }
            println!("iptables rules recovered");
        }
        return ExitCode::SUCCESS;
    }

    #[cfg(windows)]
    {
        use windows_service::service_dispatcher;

        // Try connecting to the Windows Service Control Manager.
        // If the process was started by the SCM, service_dispatcher::start
        // blocks until the service is stopped.
        match service_dispatcher::start(SERVICE_NAME, ffi_service_main) {
            Ok(()) => return ExitCode::SUCCESS,
            Err(windows_service::Error::Winapi(ref err))
                if err.raw_os_error() == Some(ERROR_FAILED_SERVICE_CONTROLLER_CONNECT) =>
            {
                // ERROR_FAILED_SERVICE_CONTROLLER_CONNECT (1063):
                // Process is running interactively from console, not under SCM.
                // Cleanly fall back to interactive console mode.
            }
            Err(error) => {
                // If SCM dispatch fails for any other reason, report warning and fall back.
                eprintln!("service dispatcher: {error}, continuing in interactive mode");
            }
        }
    }

    run_interactive(options)
}

fn run_interactive(options: cli::Options) -> ExitCode {
    #[cfg(target_os = "linux")]
    if !nix::unistd::geteuid().is_root() {
        eprintln!("Error: Portmaster requires root privileges to manage firewall and network interception. Please run with sudo or as root.");
        return ExitCode::FAILURE;
    }

    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("failed to start runtime: {error}");
            return ExitCode::FAILURE;
        }
    };

    runtime.block_on(async {
        // The log directory may not exist yet; create it before logging starts.
        if !options.log_to_stdout {
            let _ = pm_database::ensure_directory(&options.log_dir);
        }
        pm_core::logging::init(&options);
        tracing::info!(version = %cli::version(), "starting core");

        let service = match Service::start(&options).await {
            Ok(service) => std::sync::Arc::new(service),
            Err(error) => {
                tracing::error!(%error, "failed to start core");
                eprintln!("failed to start core: {error}");
                return ExitCode::FAILURE;
            }
        };

        tracing::info!(
            address = %service.local_addr(),
            "core is running"
        );

        // `run` consumes the Arc, but the flag lives in the shared Service,
        // so keep another handle to read it AFTER run() returns.
        let service_for_flag = Arc::clone(&service);
        let code = match service.run().await {
            Ok(()) => {
                if service_for_flag.restart_requested() {
                    // A restart is a graceful stop from the service's point of
                    // view, but the supervisor only spins up the new binary
                    // when we exit non-zero (see `RESTART_EXIT_CODE`). Plain
                    // shutdowns, including SIGTERM, keep exiting 0.
                    tracing::info!(code = cli::RESTART_EXIT_CODE, "restarting core");
                    ExitCode::from(cli::RESTART_EXIT_CODE as u8)
                } else {
                    ExitCode::SUCCESS
                }
            }
            Err(error) => {
                tracing::error!(%error, "core failed");
                eprintln!("core failed: {error}");
                ExitCode::FAILURE
            }
        };
        #[cfg(windows)]
        let _ = pm_winkext::force_stop_driver();
        code
    })
}

#[cfg(windows)]
fn win_service_main(_arguments: Vec<std::ffi::OsString>) {
    let res = run_service();
    let _ = pm_winkext::force_stop_driver();
    if let Err(error) = res {
        tracing::error!(%error, "windows service terminated with error");
        eprintln!("windows service terminated with error: {error}");
    }
}

#[cfg(windows)]
fn run_service() -> anyhow::Result<()> {
    use std::time::Duration;
    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

    let (stop_tx, mut stop_rx) = tokio::sync::mpsc::channel::<()>(1);

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            ServiceControl::Stop | ServiceControl::Shutdown => {
                tracing::info!("SCM control event: stop/shutdown requested");
                let _ = stop_tx.blocking_send(());
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)
        .map_err(|e| anyhow::anyhow!("failed to register service control handler: {e}"))?;

    // Tell service manager we are starting.
    status_handle
        .set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::StartPending,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::NO_ERROR,
            checkpoint: 0,
            wait_hint: Duration::from_secs(30),
            process_id: None,
        })
        .map_err(|e| anyhow::anyhow!("failed to set StartPending status: {e}"))?;

    let options = match cli::parse(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            let _ = status_handle.set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Stopped,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::Win32(1),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            });
            return Err(anyhow::anyhow!("failed to parse command line options: {error}"));
        }
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("failed to start runtime: {e}"))?;

    runtime.block_on(async {
        if !options.log_to_stdout {
            let _ = pm_database::ensure_directory(&options.log_dir);
        }
        pm_core::logging::init(&options);
        tracing::info!(version = %cli::version(), "starting PortmasterCore service");

        let service = match Service::start(&options).await {
            Ok(service) => Arc::new(service),
            Err(error) => {
                tracing::error!(%error, "failed to start core service");
                let _ = status_handle.set_service_status(ServiceStatus {
                    service_type: ServiceType::OWN_PROCESS,
                    current_state: ServiceState::Stopped,
                    controls_accepted: ServiceControlAccept::empty(),
                    exit_code: ServiceExitCode::Win32(1),
                    checkpoint: 0,
                    wait_hint: Duration::default(),
                    process_id: None,
                });
                return Err(anyhow::anyhow!("failed to start core service: {error}"));
            }
        };

        // Tell service manager we are up and running!
        let _ = status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            exit_code: ServiceExitCode::NO_ERROR,
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        });

        tracing::info!(
            address = %service.local_addr(),
            "PortmasterCore service is running"
        );

        // Spawn a background task to handle SCM stop/shutdown requests
        let stop_service = Arc::clone(&service);
        let stop_status_handle = status_handle;
        tokio::spawn(async move {
            if stop_rx.recv().await.is_some() {
                tracing::info!("SCM stop/shutdown signal received, transitioning to StopPending");
                let _ = stop_status_handle.set_service_status(ServiceStatus {
                    service_type: ServiceType::OWN_PROCESS,
                    current_state: ServiceState::StopPending,
                    controls_accepted: ServiceControlAccept::empty(),
                    exit_code: ServiceExitCode::NO_ERROR,
                    checkpoint: 0,
                    wait_hint: Duration::from_secs(30),
                    process_id: None,
                });
                stop_service.stop_token.cancel();
            }
        });

        let outcome = service.run().await;

        let exit_code = if outcome.is_ok() {
            ServiceExitCode::NO_ERROR
        } else {
            ServiceExitCode::Win32(1)
        };

        // Notify service manager that service has stopped.
        let _ = status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code,
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        });

        outcome.map_err(|e| anyhow::anyhow!("service run error: {e}"))
    })
}
