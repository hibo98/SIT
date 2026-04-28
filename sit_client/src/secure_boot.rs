use anyhow::{bail, Result};
use winreg::enums::{HKEY_LOCAL_MACHINE};
use winreg::{RegKey};
use sit_lib::secure_boot::SecureBootStatus;

pub struct SecureBoot;

impl SecureBoot {
    pub fn get_secure_boot_status() -> Result<SecureBootStatus> {
        let (updates, updates_policy) = Self::get_available_updates();
        Ok(SecureBootStatus {
            available_updates: updates,
            available_updates_policy: updates_policy,
            uefi_secure_boot_enabled: Self::get_state(),
            uefi_ca_2023_status: Self::get_uefi_ca_2023_status(),
        })
    }

    fn get_available_updates() -> (Option<u32>, Option<u32>) {
        let secure_boot = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SYSTEM\\CurrentControlSet\\Control\\SecureBoot");
        if let Ok(secure_boot) = secure_boot {
            let available_updates: Result<u32, _> = secure_boot.get_value("AvailableUpdates");
            let available_updates_policy: Result<u32, _> = secure_boot.get_value("AvailableUpdatesPolicy");
            return (available_updates.ok(), available_updates_policy.ok())
        }
        (None, None)
    }

    fn get_state() -> Option<bool> {
        let secure_boot_state = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SYSTEM\\CurrentControlSet\\Control\\SecureBoot\\State");
        if let Ok(secure_boot_state) = secure_boot_state {
            let uefi_secure_boot_enabled: Result<u32, _> = secure_boot_state.get_value("UEFISecureBootEnabled");
            return uefi_secure_boot_enabled.map_or(None, |enabled| Some(enabled == 1));
        }
        None
    }

    fn get_uefi_ca_2023_status() -> Option<String> {
        let secure_boot_servicing = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SYSTEM\\CurrentControlSet\\Control\\SecureBoot\\Servicing");
        if let Ok(secure_boot_servicing) = secure_boot_servicing {
            let uefi_ca_2023_status: Result<String, _> = secure_boot_servicing.get_value("UEFICA2023Status");
            return uefi_ca_2023_status.ok()
        }
        None
    }
}