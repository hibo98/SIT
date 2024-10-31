#[macro_use]
extern crate windows_service;

use std::io::stdin;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use anyhow::{bail, Result};
use clap::builder::NonEmptyStringValueParser;
use clap::{arg, ArgAction, Command};
use database::Database;
use job_scheduler_ng::{Job, JobScheduler};
use serde_json::json;
use wmi::{COMLibrary, WMIConnection};

use crate::config::Config;
use crate::hardware::Hardware;
use crate::licenses::Licenses;
use crate::server::Server;
use crate::software::Software;
use crate::system_status::SystemStatus;
use crate::win_os_info::OsInfo;

mod config;
mod database;
mod hardware;
mod licenses;
mod server;
mod service_mgmt;
mod software;
mod system_status;
mod win_core;
mod win_os_info;

fn internal_main(shutdown_rx: Option<Receiver<()>>) -> Result<()> {
    let mut scheduler = JobScheduler::new();
    COMLibrary::new()?;
    let db = Database::establish_connection();
    let db_update_task = db.clone();
    let db_run_tasks = db.clone();
    scheduler.add(Job::new("0 * * * * * *".parse()?, update_base_info));
    scheduler.add(Job::new("40 0/5 * * * * *".parse()?, update_rich_info));
    scheduler.add(Job::new("20 * * * * * *".parse()?, move || {
        update_task_info(db_update_task.clone());
    }));
    scheduler.add(Job::new("10 * * * * * *".parse()?, move || {
        run_tasks(db_run_tasks.clone());
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
    if let Ok(battery_status) = Hardware::get_battery_status() {
        Server::battery_status(&battery_status).unwrap();
    }
}

fn update_task_info(db: Database) {
    if let Ok(tasks) = Server::get_tasks() {
        let task_manager = db.task_manager();
        for task in tasks {
            task_manager.add_new_task(task.clone()).unwrap();
        }
    }
}

fn run_tasks(db: Database) {
    for task in db.task_manager().get_pending_tasks().unwrap() {
        let task_manager = db.task_manager();
        task_manager.task_update_running(&task);
        thread::spawn(move || {
            let task_name = task.task.get("name").map(|v| v.as_str().unwrap_or_default()).unwrap_or_default();
            if task_name.eq_ignore_ascii_case("delete-user-profile") {
                let parameters = task.task.get("parameters").map(|v| v.as_object()).unwrap_or_default();
                if let Some(parameters) = parameters {
                    let sid = parameters.get("sid").map(|p| p.as_str().unwrap_or_default()).unwrap_or_default();
                    let result = OsInfo::delete_user_profile(sid);
                    if result.is_ok() {
                        task_manager.task_update_successful(&task, None);
                    } else if let Err(e) = result {
                        task_manager.task_update_failed(&task, Some(json!({"error": e.to_string()})));
                    }
                }
                task_manager.task_update_failed(&task, Some(json!({"error": "missing parameters"})));
            }
            task_manager.task_update_failed(&task, Some(json!({"error": "unkown task"})));
        });
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
            } else if func == *"delete-user-profile" {
                println!("Enter SID of User: ");
                let mut buffer = String::new();

                stdin().read_line(&mut buffer)?;
                let res = match buffer.trim_end() {
                    "" => bail!("Please enter a vaild SID for the user"),
                    sid => OsInfo::delete_user_profile(sid),
                };
                println!("{:#?}", res);
            } else if func == *"power-status" {
                Hardware::get_battery_status()?;
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
