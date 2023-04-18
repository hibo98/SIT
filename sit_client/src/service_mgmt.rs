use anyhow::Result;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use windows_service::service::{
    Service, ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl,
    ServiceExitCode, ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::ServiceControlHandlerResult;
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use windows_service::{service_control_handler, service_dispatcher};

pub const SERVICE_NAME: &str = "SitClientService";

define_windows_service!(sit_service_main, service_main);

fn get_service_handle(service_access: ServiceAccess) -> Result<Service> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
    let service = manager.open_service(SERVICE_NAME, service_access)?;
    Ok(service)
}

pub fn install_service(service_path: &Path) -> Result<()> {
    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from("S-IT Client Service"),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_path.to_path_buf(),
        launch_arguments: vec!["start".into(), "-s".into()],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    let service = get_service_handle(
        ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::CHANGE_CONFIG,
    );
    let service_handle = if let Ok(service_handle) = service {
        if service_handle.query_status()?.current_state != ServiceState::Stopped {
            service_handle.stop()?;
        }
        service_handle.change_config(&service_info)?;
        service_handle
    } else {
        let manager =
            ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;
        manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?
    };
    service_handle.set_delayed_auto_start(true)?;
    Ok(())
}

pub fn uninstall_service() -> Result<()> {
    let service = get_service_handle(
        ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE,
    )?;
    if service.query_status()?.current_state != ServiceState::Stopped {
        service.stop()?;
    }
    service.delete()?;
    Ok(())
}

pub fn start_service() -> Result<()> {
    let service = get_service_handle(ServiceAccess::QUERY_STATUS | ServiceAccess::START)?;
    if service.query_status()?.current_state != ServiceState::Running {
        service.start(&[OsStr::new("")])?;
    }
    Ok(())
}

pub fn stop_service() -> Result<()> {
    let service = get_service_handle(ServiceAccess::QUERY_STATUS | ServiceAccess::STOP)?;
    if service.query_status()?.current_state != ServiceState::Stopped {
        service.stop()?;
    }
    Ok(())
}

pub fn run_service_main() -> Result<()> {
    service_dispatcher::start(SERVICE_NAME, sit_service_main)?;

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
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
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

    crate::internal_main(Some(shutdown_rx)).unwrap();

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
