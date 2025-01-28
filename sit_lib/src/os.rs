use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct WinOsInfo {
    pub operating_system: String,
    pub os_version: String,
    pub computer_name: String,
    pub domain: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfiles {
    pub profiles: Vec<ProfileInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileInfo {
    #[serde(default)]
    pub domain: Option<String>,
    pub username: Option<String>,
    pub sid: String,
    pub health_status: u8,
    pub roaming_configured: bool,
    pub roaming_path: Option<String>,
    pub roaming_preference: Option<bool>,
    pub last_use_time: Option<DateTime<FixedOffset>>,
    pub last_download_time: Option<DateTime<FixedOffset>>,
    pub last_upload_time: Option<DateTime<FixedOffset>>,
    pub status: u32,
    pub size: Option<u64>,
    pub path_size: Option<Vec<PathInfo>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathInfo {
    pub path: String,
    pub size: u64,
}
