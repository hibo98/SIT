#[macro_use]
extern crate windows_service;

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::{env, thread};

use anyhow::Result;
use clap::builder::NonEmptyStringValueParser;
use clap::{arg, value_parser, ArgAction, Command};
use job_scheduler_ng::{Job, JobScheduler};
use wmi::{COMLibrary, WMIConnection};

use crate::config::Config;
use crate::hardware::Hardware;
use crate::licenses::Licenses;
use crate::server::Server;
use crate::software::Software;
use crate::system_status::SystemStatus;
use crate::win_os_info::OsInfo;

mod config;
mod hardware;
mod licenses;
mod server;
mod service_mgmt;
mod software;
mod system_status;
mod win_os_info;

fn internal_main(shutdown_rx: Option<Receiver<()>>) -> Result<()> {
    let mut scheduler = JobScheduler::new();
    COMLibrary::new()?;
    scheduler.add(Job::new("0 * * * * * *".parse().unwrap(), update_base_info));
    scheduler.add(Job::new(
        "0 0/5 * * * * *".parse().unwrap(),
        update_rich_info,
    ));

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

fn update_base_info() {
    let com_con = COMLibrary::without_security().unwrap();
    let wmi_con = WMIConnection::new(com_con).unwrap();
    let os_info = OsInfo::get_os_info(&wmi_con);
    if let Ok(os_info) = os_info {
        Server::register(&os_info.computer_name).unwrap();
        Server::os(&os_info).unwrap();
    }
}

fn update_rich_info() {
    let com_con = COMLibrary::without_security().unwrap();
    let wmi_con = WMIConnection::new(com_con).unwrap();
    if let Ok(hardware_info) = Hardware::get_hardware_info(&wmi_con) {
        Server::hardware(&hardware_info).unwrap();
    }
    if let Ok(profiles) = OsInfo::get_user_profiles(&wmi_con) {
        Server::profiles(&profiles).unwrap();
    }
    let software_lib = Software::get_software_list();
    Server::software(&software_lib).unwrap();
    if let Ok(volumes) = SystemStatus::get_volume_status(&wmi_con) {
        Server::status_volumes(&volumes).unwrap();
    }
    if let Ok(licenses) = Licenses::collect_licenses() {
        Server::licenses(&licenses).unwrap();
    }
}

fn cli() -> Command {
    Command::new("client")
        .subcommand_required(true)
        .subcommand(
            Command::new("start").about("Start client routine").arg(
                arg!(-s --service "Starts client routine as windows service")
                    .action(ArgAction::SetTrue),
            ),
        )
        .subcommand(
            Command::new("install-service")
                .about("Installs windows service, if already installed updates configuration")
                .arg(
                    arg!(-p --path <PATH> "Path to the executable")
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .subcommand(Command::new("uninstall-service").about("Uninstalls windows service"))
        .subcommand(Command::new("start-service").about("Start windows service"))
        .subcommand(Command::new("stop-service").about("Stop windows service"))
        .subcommand(
            Command::new("debug")
                .about("Executes functions for debug reasons")
                .arg(
                    arg!(-f --function <FUNCTION> "Function to execute")
                        .value_parser(NonEmptyStringValueParser::new())
                        .required(true),
                ),
        )
        .subcommand(Command::new("update").about("Update info on server").args([
            arg!(-b --base "Update only base info").action(ArgAction::SetTrue),
            arg!(-r --rich "Update only rich info").action(ArgAction::SetTrue),
        ]))
}

fn main() -> Result<()> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("start", sub_matches)) => {
            let service = sub_matches.get_one::<bool>("service");
            Config::setup()?;
            if let Some(true) = service {
                service_mgmt::run_service_main()?;
            } else {
                internal_main(None)?;
            }
        }
        Some(("install-service", sub_matches)) => {
            let path = sub_matches.get_one::<PathBuf>("path");
            if let Some(path) = path {
                service_mgmt::install_service(path)?;
            } else {
                service_mgmt::install_service(&env::current_exe()?)?;
            }
        }
        Some(("uninstall-service", _)) => {
            service_mgmt::uninstall_service()?;
        }
        Some(("start-service", _)) => {
            service_mgmt::start_service()?;
        }
        Some(("stop-service", _)) => {
            service_mgmt::stop_service()?;
        }
        Some(("debug", sub_matches)) => {
            let com_con = COMLibrary::new()?;
            let wmi_con = WMIConnection::new(com_con)?;
            let func = sub_matches
                .get_one::<String>("function")
                .cloned()
                .unwrap_or_default();
            if func == *"hardware-info" {
                println!("{:#?}", Hardware::get_hardware_info(&wmi_con));
            } else if func == *"software-list" {
                println!("{:#?}", Software::get_software_list());
            } else if func == *"os-info" {
                println!("{:#?}", OsInfo::get_os_info(&wmi_con));
            } else if func == *"user-profiles" {
                println!("{:#?}", OsInfo::get_user_profiles(&wmi_con));
            } else if func == *"windows-key" {
                println!("{:#?}", Licenses::get_windows_key());
            } else if func == *"system-status" {
                println!("{:#?}", SystemStatus::get_volume_status(&wmi_con));
            }
        }
        Some(("update", sub_matches)) => {
            COMLibrary::new()?;
            if let Some(true) = sub_matches.get_one::<bool>("base") {
                update_base_info();
            } else if let Some(true) = sub_matches.get_one::<bool>("rich") {
                update_rich_info();
            } else {
                update_base_info();
                update_rich_info();
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
