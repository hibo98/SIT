use std::path::Path;
use anyhow::Result;
use reqwest::blocking::Client;
use sit_lib::hardware::{BatteryStatus, HardwareInfoV2};
use sit_lib::licenses::LicenseBundle;
use sit_lib::os::{UserProfiles, WinOsInfo};
use sit_lib::server::Register;
use sit_lib::software::SoftwareLibrary;
use sit_lib::system_status::VolumeList;
use sit_lib::task::{Task, TaskBundle, TaskUpdate};

use crate::Config;

pub struct Server;

impl Server {
    pub fn register(name: &str) -> Result<()> {
        let request = Self::build_client()?
            .post(format!("{}/api/v1/register", Config::get_web_api()?))
            .json(&Register {
                name: name.to_string(),
                uuid: Config::get_uuid()?,
            })
            .send();
        if let Ok(request) = request {
            if request.status().is_success() {
                let register: Result<Register, _> = request.json();
                Config::set_uuid(register?.uuid.unwrap())?;
            } else {
                println!("R:: {:#?}", request.status().as_u16());
            }
        }
        Ok(())
    }

    pub fn os(os_info: &WinOsInfo) -> Result<()> {
        let _request = Self::build_client()?
            .post(format!(
                "{}/api/v1/os/{}",
                Config::get_web_api()?,
                Config::get_uuid()?.unwrap()
            ))
            .json(os_info)
            .send();
        Ok(())
    }

    pub fn hardware(hardware_info: &HardwareInfoV2) -> Result<()> {
        let _request = Self::build_client()?
            .post(format!(
                "{}/api/v2/hardware/{}",
                Config::get_web_api()?,
                Config::get_uuid()?.unwrap()
            ))
            .json(hardware_info)
            .send();
        Ok(())
    }

    pub fn software(software_lib: &SoftwareLibrary) -> Result<()> {
        let _request = Self::build_client()?
            .post(format!(
                "{}/api/v1/software/{}",
                Config::get_web_api()?,
                Config::get_uuid()?.unwrap()
            ))
            .json(software_lib)
            .send();
        Ok(())
    }

    pub fn profiles(profiles: &UserProfiles) -> Result<()> {
        let _request = Self::build_client()?
            .post(format!(
                "{}/api/v1/profiles/{}",
                Config::get_web_api()?,
                Config::get_uuid()?.unwrap()
            ))
            .json(profiles)
            .send();
        Ok(())
    }

    pub fn licenses(licenses: &LicenseBundle) -> Result<()> {
        let _request = Self::build_client()?
            .post(format!(
                "{}/api/v1/licenses/{}",
                Config::get_web_api()?,
                Config::get_uuid()?.unwrap()
            ))
            .json(licenses)
            .send();
        Ok(())
    }

    pub fn status_volumes(volumes: &VolumeList) -> Result<()> {
        let _request = Self::build_client()?
            .post(format!(
                "{}/api/v1/status/{}/volumes",
                Config::get_web_api()?,
                Config::get_uuid()?.unwrap()
            ))
            .json(volumes)
            .send();
        Ok(())
    }

    pub fn battery_status(battery_status: &BatteryStatus) -> Result<()> {
        let _request = Self::build_client()?
            .post(format!(
                "{}/api/v1/status/{}/battery",
                Config::get_web_api()?,
                Config::get_uuid()?.unwrap()
            ))
            .json(battery_status)
            .send();
        Ok(())
    }

    pub fn get_tasks() -> Result<Vec<Task>> {
        let response = Self::build_client()?
            .get(format!(
                "{}/api/v1/tasks/{}",
                Config::get_web_api()?,
                Config::get_uuid()?.unwrap()
            ))
            .send()?;
        let task_bundle: TaskBundle = response.json()?;
        Ok(task_bundle.tasks)
    }

    pub fn update_task(task_update: &TaskUpdate) -> Result<()> {
        let _request = Self::build_client()?
            .post(format!(
                "{}/api/v1/tasks/{}",
                Config::get_web_api()?,
                Config::get_uuid()?.unwrap()
            ))
            .json(task_update)
            .send();
        Ok(())
    }

    fn build_client() -> Result<Client> {
        let string_path = Config::get_ca_path()?;
        let path = Path::new(&string_path);
        let der = std::fs::read(path)?;
        let cert = reqwest::Certificate::from_der(&der)?;
        Ok(Client::builder()
            .add_root_certificate(cert)
            .https_only(true)
            .build()?)
    }
}
