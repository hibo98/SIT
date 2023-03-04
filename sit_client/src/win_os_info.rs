#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use anyhow::{bail, Result};
use serde::Deserialize;
use sit_lib::os::{ProfileInfo, UserProfiles, WinOsInfo};
use std::ffi::{c_void, CString};
use std::ptr;
use walkdir::WalkDir;
use widestring::U16CString;
use winapi::shared::sddl::ConvertStringSidToSidA;
use winapi::shared::winerror::{ERROR_INVALID_PARAMETER, ERROR_INVALID_SID, ERROR_NONE_MAPPED};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::LocalFree;
use winapi::um::winbase::LookupAccountSidW;
use winapi::um::winnt::SID_NAME_USE;
use winreg::enums::HKEY_LOCAL_MACHINE;
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

#[derive(Debug)]
struct AccountInfo {
    username: String,
    domain_name: String,
}

struct WinPointer {
    inner: *mut c_void,
}

impl Drop for WinPointer {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.inner);
        }
    }
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
                username: OsInfo::lookup_account_by_sid(&up.SID)
                    .ok()
                    .map(|account| format!("{}\\{}", account.domain_name, account.username)),
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

    #[allow(unused_assignments)]
    fn lookup_account_by_sid(sid_str: &str) -> Result<AccountInfo> {
        let sid_c_string = CString::new(sid_str)?;
        let mut sid_ptr = WinPointer {
            inner: ptr::null_mut(),
        };

        unsafe {
            if ConvertStringSidToSidA(sid_c_string.as_ptr(), &mut sid_ptr.inner) == 0 {
                let err = GetLastError();
                if err == ERROR_INVALID_PARAMETER {
                    bail!("Conversion of String to PSID failed (ERROR_INVALID_PARAMETER) SID: {sid_str}");
                } else if err == ERROR_INVALID_SID {
                    bail!("Conversion of String to PSID failed (ERROR_INVALID_SID) SID: {sid_str}");
                } else {
                    bail!("Conversion of String to PSID failed ({err}) SID: {sid_str}");
                }
            }
        }

        let mut name: [u16; 256] = [0; 256];
        let mut name_size = name.len() as u32;
        let mut domain_name: [u16; 256] = [0; 256];
        let mut domain_name_size = domain_name.len() as u32;
        let mut sid_name_use: SID_NAME_USE = 0;

        unsafe {
            if LookupAccountSidW(
                ptr::null(),
                sid_ptr.inner,
                name.as_mut_ptr(),
                &mut name_size,
                domain_name.as_mut_ptr(),
                &mut domain_name_size,
                &mut sid_name_use,
            ) == 0
            {
                let err = GetLastError();
                if err == ERROR_NONE_MAPPED {
                    bail!("Lookup of Account failed (ERROR_NONE_MAPPED)");
                } else {
                    bail!("Lookup of Account failed ({err})");
                }
            }

            let username = U16CString::from_ptr_str(name.as_ptr()).to_string_lossy();
            let domain_name = U16CString::from_ptr_str(domain_name.as_ptr()).to_string_lossy();

            Ok(AccountInfo {
                username,
                domain_name,
            })
        }
    }

    fn get_dir_size(path: &String) -> Result<u64> {
        Ok(WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|f| f.metadata().map_or(0, |f| f.len()))
            .sum())
    }

    fn get_ubr() -> Result<u32> {
        let sub_key = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")?;
        Ok(sub_key.get_value("UBR")?)
    }
}
