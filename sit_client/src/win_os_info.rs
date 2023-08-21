#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use anyhow::Result;
use serde::Deserialize;
use sit_lib::os::{PathInfo, ProfileInfo, UserProfiles, WinOsInfo};
use std::ffi::CString;
use walkdir::WalkDir;
use windows::core::{PCSTR, PCWSTR, PWSTR};
use windows::Win32::Foundation::{LocalFree, HLOCAL, PSID};
use windows::Win32::Security::Authorization::ConvertStringSidToSidA;
use windows::Win32::Security::{LookupAccountSidW, SID_NAME_USE};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;
use wmi::{WMIConnection, WMIDateTime, WMIError};

pub struct OsInfo;

#[derive(Deserialize, Debug)]
struct Win32_OperatingSystem {
    // Name of Operating System
    Caption: String,
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

#[derive(Debug)]
struct AccountInfo {
    username: String,
    domain_name: String,
}

struct WinPointer {
    inner: PSID,
}

impl Drop for WinPointer {
    fn drop(&mut self) {
        unsafe {
            let _ = LocalFree(HLOCAL(self.inner.0));
        }
    }
}

impl OsInfo {
    pub fn get_os_info(wmi_con: &WMIConnection) -> Result<WinOsInfo, WMIError> {
        let win32_cs: Vec<Win32_ComputerSystem> = wmi_con.query()?;
        let win32_os: Vec<Win32_OperatingSystem> = wmi_con.query()?;
        if let Some(win32_os) = win32_os.last() {
            if let Some(win32_cs) = win32_cs.last() {
                return Ok(WinOsInfo {
                    operating_system: win32_os.Caption.clone(),
                    os_version: Self::get_windows_version().unwrap_or_default(),
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
            .map(|up| {
                let account_info = OsInfo::lookup_account_by_sid(&up.SID).ok();
                ProfileInfo {
                    domain: account_info.as_ref().map(|a| a.domain_name.clone()),
                    username: account_info
                        .as_ref()
                        .map(|account| account.username.clone()),
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
                    path_size: if up.Loaded {
                        None
                    } else {
                        OsInfo::get_profile_dir_path_infos(&up.LocalPath).ok()
                    },
                }
            })
            .collect();
        Ok(UserProfiles { profiles: vec })
    }

    fn lookup_account_by_sid(sid_str: &str) -> Result<AccountInfo> {
        let sid_c_string = CString::new(sid_str)?;
        let mut sid_ptr = WinPointer {
            inner: PSID::default(),
        };

        unsafe {
            ConvertStringSidToSidA(
                PCSTR::from_raw(sid_c_string.as_ptr() as *const u8),
                &mut sid_ptr.inner,
            )?;
        }

        let mut name: [u16; 256] = [0; 256];
        let mut name_size = name.len() as u32;
        let name_pwstr = PWSTR::from_raw(name.as_mut_ptr());
        let mut domain_name: [u16; 256] = [0; 256];
        let mut domain_name_size = domain_name.len() as u32;
        let domain_name_pwstr = PWSTR::from_raw(domain_name.as_mut_ptr());
        let mut sid_name_use = SID_NAME_USE::default();

        unsafe {
            LookupAccountSidW(
                PCWSTR::null(),
                sid_ptr.inner,
                name_pwstr,
                &mut name_size,
                domain_name_pwstr,
                &mut domain_name_size,
                &mut sid_name_use,
            )?;

            Ok(AccountInfo {
                username: name_pwstr.to_string()?,
                domain_name: domain_name_pwstr.to_string()?,
            })
        }
    }

    fn get_profile_dir_path_infos(path: &String) -> Result<Vec<PathInfo>> {
        Ok(vec!["AppData\\Roaming"]
            .into_iter()
            .map(|sub_path| {
                let path = path.to_owned() + "\\" + sub_path;
                let size = OsInfo::get_dir_size(&path).unwrap_or_default();
                PathInfo { path, size }
            })
            .collect())
    }

    fn get_dir_size(path: &String) -> Result<u64> {
        Ok(WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|f| f.metadata().map_or(0, |f| f.len()))
            .sum())
    }

    fn get_windows_version() -> Result<String> {
        let win_curr_ver = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")?;
        let major: u32 = win_curr_ver.get_value("CurrentMajorVersionNumber")?;
        let minor: u32 = win_curr_ver.get_value("CurrentMinorVersionNumber")?;
        let build: String = win_curr_ver.get_value("CurrentBuild")?;
        let ubr: u32 = win_curr_ver.get_value("UBR")?;
        Ok(format!("{}.{}.{}.{}", major, minor, build, ubr))
    }
}
