#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use anyhow::{bail, Result};
use serde::Deserialize;
use sit_lib::os::{ProfileInfo, UserProfiles, WinOsInfo};
use std::ffi::CString;
use walkdir::WalkDir;
use windows::core::{PCSTR, PCWSTR, PWSTR};
use windows::Win32::Foundation::{GetLastError, PSID, HLOCAL};
use windows::Win32::Security::Authorization::ConvertStringSidToSidA;
use windows::Win32::Security::{LookupAccountSidW, SID_NAME_USE};
use windows::Win32::System::Memory::LocalFree;
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
            let _ = LocalFree(HLOCAL(self.inner.0 as isize));
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

    pub fn get_windows_key() -> Result<String> {
        let mut key_output = String::new();
        let chars = vec![
            "B", "C", "D", "F", "G", "H", "J", "K", "M", "P", "Q", "R", "T", "V", "W", "X", "Y",
            "2", "3", "4", "6", "7", "8", "9",
        ];
        let key_offset = 52;

        let win_curr_ver = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")?;
        let mut key: Vec<u8> = win_curr_ver.get_raw_value("DigitalProductId")?.bytes;

        let is_win8 = (key[66] as f64 / 6_f64).ceil() as u8 & 1;
        key[66] = (key[66] & 0xf7) | ((is_win8 & 2) * 4);

        let mut last = 0;

        for _i in (0..25).rev() {
            let mut cur: u32 = 0;
            for x in (0..15).rev() {
                cur *= 256;
                cur += key[x + key_offset] as u32;
                key[x + key_offset] = (cur as f64 / 24_f64).floor() as u8;
                cur %= 24;
            }
            key_output = chars[cur as usize].to_owned() + &key_output;
            last = cur;
        }

        let key_part1 = &key_output.clone()[1..(1 + last) as usize];
        let key_part2 = &key_output.clone()[1..(key_output.len())];
        
        if last == 0 {
            key_output = "N".to_owned() + key_part2;
        } else {
            let ins_pos = key_part2.find(key_part1).unwrap_or_default() + key_part1.len();
            let mut key_part2 = key_part2.to_owned();
            key_part2.insert(ins_pos, 'N');
            key_output = key_part2;
        }
        key_output.insert(5, '-');
        key_output.insert(11, '-');
        key_output.insert(17, '-');
        key_output.insert(23, '-');

        Ok(key_output)
    }

    fn lookup_account_by_sid(sid_str: &str) -> Result<AccountInfo> {
        let sid_c_string = CString::new(sid_str)?;
        let mut sid_ptr = WinPointer {
            inner: PSID::default(),
        };

        unsafe {
            if !ConvertStringSidToSidA(
                PCSTR::from_raw(sid_c_string.as_ptr() as *const u8),
                &mut sid_ptr.inner,
            )
            .as_bool()
            {
                let err = GetLastError().to_hresult().message();
                bail!("Conversion of String to PSID failed ({err}) SID: {sid_str}");
            }
        }

        let mut name: [u16; 256] = [0; 256];
        let mut name_size = name.len() as u32;
        let name_pwstr = PWSTR::from_raw(name.as_mut_ptr());
        let mut domain_name: [u16; 256] = [0; 256];
        let mut domain_name_size = domain_name.len() as u32;
        let domain_name_pwstr = PWSTR::from_raw(domain_name.as_mut_ptr());
        let mut sid_name_use = SID_NAME_USE::default();

        unsafe {
            if !LookupAccountSidW(
                PCWSTR::null(),
                sid_ptr.inner,
                name_pwstr,
                &mut name_size,
                domain_name_pwstr,
                &mut domain_name_size,
                &mut sid_name_use,
            )
            .as_bool()
            {
                let err = GetLastError().to_hresult().message();
                bail!("Lookup of Account failed ({err})");
            }

            Ok(AccountInfo {
                username: name_pwstr.to_string()?,
                domain_name: domain_name_pwstr.to_string()?,
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
