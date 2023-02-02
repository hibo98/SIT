use anyhow::Result;
use sit_lib::hardware::HardwareInfo;
use sit_lib::os::{UserProfiles, WinOsInfo};
use sit_lib::server::Register;
use sit_lib::software::SoftwareLibrary;

use crate::Config;

pub struct Server;

impl Server {
    pub fn register(name: &str) -> Result<()> {
        let request = reqwest::blocking::Client::new()
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
        let _request = reqwest::blocking::Client::new()
            .post(format!(
                "{}/api/v1/os/{}",
                Config::get_web_api()?,
                Config::get_uuid()?.unwrap()
            ))
            .json(os_info)
            .send();
        Ok(())
    }

    pub fn hardware(hardware_info: &HardwareInfo) -> Result<()> {
        let _request = reqwest::blocking::Client::new()
            .post(format!(
                "{}/api/v1/hardware/{}",
                Config::get_web_api()?,
                Config::get_uuid()?.unwrap()
            ))
            .json(hardware_info)
            .send();
        Ok(())
    }

    pub fn software(software_lib: &SoftwareLibrary) -> Result<()> {
        let _request = reqwest::blocking::Client::new()
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
        let _request = reqwest::blocking::Client::new()
            .post(format!(
                "{}/api/v1/profiles/{}",
                Config::get_web_api()?,
                Config::get_uuid()?.unwrap()
            ))
            .json(profiles)
            .send();
        Ok(())
    }
}
