#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use anyhow::Result;
use serde::Deserialize;
use sit_lib::system_status::{Volume, VolumeList};
use wmi::WMIConnection;

#[derive(Deserialize, Debug)]
struct Win32_Volume {
    Capacity: u64,
    DriveLetter: Option<String>,
    DriveType: u32,
    FileSystem: String,
    FreeSpace: u64,
    Label: Option<String>,
}

pub struct SystemStatus;
impl SystemStatus {
    pub fn get_volume_status(wmi_con: &WMIConnection) -> Result<VolumeList> {
        let mut volumes = Vec::new();
        let win32_v: Vec<Win32_Volume> = wmi_con.query()?;
        for v in win32_v {
            if v.DriveType != 3 {
                continue;
            }
            if let Some(drive_letter) = v.DriveLetter {
                volumes.push(Volume {
                    drive_letter,
                    label: v.Label,
                    file_system: v.FileSystem,
                    capacity: v.Capacity,
                    free_space: v.FreeSpace,
                });
            }
        }
        Ok(VolumeList { volumes })
    }
}
