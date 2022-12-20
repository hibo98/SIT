#[macro_use]
extern crate windows_service;

use std::{thread, env};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::sync::{mpsc};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use anyhow::Result;
use clap::{arg, value_parser, Command, ArgAction};
use job_scheduler::{Job, JobScheduler};
use windows_service::{service_control_handler, service_dispatcher};
use windows_service::service::{ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode, ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use windows_service::service_control_handler::ServiceControlHandlerResult;
use wmi::{COMLibrary, WMIConnection};

use crate::config::Config;
use crate::hardware::Hardware;
use crate::server::Server;
use crate::software::Software;
use crate::win_os_info::OsInfo;

mod win_os_info;
mod hardware;
mod software;
mod config;
mod server;

const SERVICE_NAME: &str = "SitClientService";

define_windows_service!(sit_service_main, service_main);

fn install_service(service_path: &PathBuf) -> windows_service::Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from("S-IT Client Service"),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_path.clone(),
        launch_arguments: vec!["start".into(), "-s".into()],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    manager.create_service(&service_info, ServiceAccess::QUERY_STATUS)?;

    Ok(())
}

fn uninstall_service() -> windows_service::Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;

    let service = manager.open_service(SERVICE_NAME, ServiceAccess::DELETE)?;
    service.delete()?;

    Ok(())
}

fn start_service() -> windows_service::Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;

    let service = manager.open_service(SERVICE_NAME, ServiceAccess::START)?;
    service.start(&[OsStr::new("")])?;

    Ok(())
}

fn stop_service() -> windows_service::Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;

    let service = manager.open_service(SERVICE_NAME, ServiceAccess::STOP)?;
    service.stop()?;

    Ok(())
}

fn service_main(arguments: Vec<OsString>) {
    if let Err(_e) = run_service(arguments) {
        // Handle error in some way.
    }
}

fn run_service(_arguments: Vec<OsString>) -> windows_service::Result<()> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Interrogate => {
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Stop => {
                shutdown_tx.send(()).unwrap();
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Tell the system that the service is running now
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    internal_main(Some(shutdown_rx)).unwrap();

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

fn internal_main(shutdown_rx: Option<Receiver<()>>) -> Result<()> {
    let mut scheduler = JobScheduler::new();
    let com_con = COMLibrary::new()?;
    let wmi_con1 = WMIConnection::new(com_con)?;
    let wmi_con2 = WMIConnection::new(com_con)?;
    scheduler.add(Job::new("0 * * * * * *".parse().unwrap(), move || {
        let os_info = OsInfo::get_os_info(&wmi_con1);
        if let Ok(os_info) = os_info {
            Server::register(&os_info.computer_name).unwrap();
            Server::os(&os_info).unwrap();
        }
    }));
    scheduler.add(Job::new("0 1/5 * * * * *".parse().unwrap(), move || {
        let hardware_info = Hardware::get_hardware_info(&wmi_con2);
        if let Ok(hardware_info) = hardware_info {
            Server::hardware(&hardware_info).unwrap();
        }
        let profiles = OsInfo::get_user_profiles(&wmi_con2);
        if let Ok(profiles) = profiles {
            Server::profiles(&profiles).unwrap();
        }
        let software_lib = Software::get_software_list();
        Server::software(&software_lib).unwrap();
    }));

    if let Some(shutdown_rx) = shutdown_rx {
        loop {
            scheduler.tick();

            match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
                // Break the loop either upon stop or channel disconnect
                Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,

                // Continue work if no events were received within the timeout
                Err(mpsc::RecvTimeoutError::Timeout) => (),
            };
        }
    } else {
        loop {
            scheduler.tick();

            thread::sleep(Duration::from_secs(1));
        }
    }

    Ok(())
}

fn cli() -> Command {
    Command::new("client")
        .subcommand_required(true)
        .subcommand(
            Command::new("start")
                .about("Start client routine")
                .arg(
                    arg!(-s --service "Starts client routine as windows service")
                        .action(ArgAction::SetTrue)
                ),
        )
        .subcommand(
            Command::new("install-service")
                .about("Installs Windows Service")
                .arg(
                    arg!(-p --path <PATH> "Path to the executable")
                        .value_parser(value_parser!(PathBuf))
                )
        )
        .subcommand(
            Command::new("uninstall-service")
                .about("Uninstalls Windows Service")
        )
        .subcommand(
            Command::new("start-service")
                .about("Start Windows Service")
        )
        .subcommand(
            Command::new("stop-service")
                .about("Stop Windows Service")
        )
}

fn main() -> Result<()> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("start", sub_matches)) => {
            let service = sub_matches.get_one::<bool>("service");
            Config::setup()?;
            if let Some(true) = service {
                service_dispatcher::start(SERVICE_NAME, sit_service_main)?;
            } else {
                internal_main(None)?;
            }
        },
        Some(("install-service", sub_matches)) => {
            let path = sub_matches.get_one::<PathBuf>("path");
            if let Some(path) = path {
                install_service(path)?;
            } else {
                let exe = &env::current_exe()?;
                install_service(exe)?;
            }
        },
        Some(("uninstall-service", _)) => {
            uninstall_service()?;
        },
        Some(("start-service", _)) => {
            start_service()?;
        },
        Some(("stop-service", _)) => {
            stop_service()?;
        },
        _ => unreachable!(),
    }
    Ok(())
}
