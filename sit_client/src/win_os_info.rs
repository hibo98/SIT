#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use anyhow::Result;
use powershell_script::PsScriptBuilder;
use serde::Deserialize;
use sit_lib::os::{ProfileInfo, UserProfiles, WinOsInfo};
use walkdir::WalkDir;
use winreg::enums::{HKEY_LOCAL_MACHINE};
use winreg::RegKey;
use wmi::{WMIConnection, WMIDateTime, WMIError};

pub struct OsInfo;

#[derive(Deserialize, Debug)]
struct Win32_OperatingSystem {
    // Name of Operating System
    Caption: String,
    // Version of Operating System
    Version: String,
}

#[derive(Deserialize, Debug)]
struct Win32_ComputerSystem {
    // Name of Computer
    DNSHostName: String,
    // Domain of Computer
    Domain: String,
}

#[derive(Deserialize, Debug)]
struct Win32_UserProfile {
    SID: String,
    HealthStatus: u8,
    RoamingConfigured: bool,
    RoamingPath: Option<String>,
    RoamingPreference: Option<bool>,
    LastUseTime: WMIDateTime,
    LastDownloadTime: Option<WMIDateTime>,
    LastUploadTime: Option<WMIDateTime>,
    Status: u32,
    Special: bool,
    LocalPath: String,
    Loaded: bool,
}

impl OsInfo {
    pub fn get_os_info(wmi_con: &WMIConnection) -> Result<WinOsInfo, WMIError> {
        let win32_cs: Vec<Win32_ComputerSystem> = wmi_con.query()?;
        let win32_os: Vec<Win32_OperatingSystem> = wmi_con.query()?;
        if let Some(win32_os) = win32_os.last() {
            if let Some(win32_cs) = win32_cs.last() {
                let ubr = Self::get_ubr();
                let version = if let Ok(ubr) = ubr {
                    format!("{}.{}", win32_os.Version, ubr)
                } else {
                    win32_os.Version.clone()
                };
                return Ok(WinOsInfo {
                    operating_system: win32_os.Caption.clone(),
                    os_version: version,
                    computer_name: win32_cs.DNSHostName.clone(),
                    domain: win32_cs.Domain.clone(),
                });
            }
        }
        Err(WMIError::ResultEmpty)
    }

    pub fn get_user_profiles(wmi_con: &WMIConnection) -> Result<UserProfiles, WMIError> {
        let win32_up: Vec<Win32_UserProfile> = wmi_con.query()?;
        let vec = win32_up
            .iter()
            .filter(|up| !up.Special)
            .filter(|up| up.SID.starts_with("S-1-5-21-"))
            .map(|up| ProfileInfo {
                username: OsInfo::lookup_sid(&up.SID).ok(),
                sid: up.SID.clone(),
                health_status: up.HealthStatus,
                roaming_configured: up.RoamingConfigured,
                roaming_path: up.RoamingPath.clone(),
                roaming_preference: up.RoamingPreference,
                last_use_time: up.LastUseTime.0,
                last_download_time: up.LastDownloadTime.as_ref().map(|ts| ts.0),
                last_upload_time: up.LastUploadTime.as_ref().map(|ts| ts.0),
                status: up.Status,
                size: if up.Loaded {
                    None
                } else {
                    OsInfo::get_dir_size(&up.LocalPath).ok()
                },
            })
            .collect();
        Ok(UserProfiles { profiles: vec })
    }

    fn lookup_sid(sid: &String) -> Result<String> {
        let shell = PsScriptBuilder::new()
            .no_profile(true)
            .non_interactive(true)
            .hidden(true)
            .print_commands(false)
            .build();
        let output = shell.run(
            format!(
                r#"$SID ='{sid}'
$objSID = New-Object System.Security.Principal.SecurityIdentifier($SID)
$objUser = $objSID.Translate([System.Security.Principal.NTAccount])
Write-Host $objUser.Value"#
            )
            .as_str(),
        )?;
        let username = output.stdout().unwrap().trim().to_string();
        Ok(username)
    }

    fn get_dir_size(path: &String) -> Result<u64> {
        Ok(WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|f| f.metadata().map_or(0, |f| f.len()))
            .sum())
    }

    fn get_ubr() -> Result<u32> {
        let sub_key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")?;
        Ok(sub_key.get_value("UBR")?)
    }
}
