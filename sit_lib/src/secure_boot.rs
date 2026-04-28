use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SecureBootStatus {
    pub available_updates: Option<u32>,
    pub available_updates_policy: Option<u32>,
    pub uefi_secure_boot_enabled: Option<bool>,
    pub uefi_ca_2023_status: Option<String>,
}