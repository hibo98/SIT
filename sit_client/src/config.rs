use anyhow::Result;
use uuid::Uuid;
use winreg::enums::{KEY_ALL_ACCESS, KEY_WRITE};
use winreg::HKLM;

pub struct Config;

impl Config {
    pub fn setup() -> Result<()> {
        let software = HKLM.open_subkey_with_flags("SOFTWARE", KEY_ALL_ACCESS)?;
        let schkola_ggmbh = software.create_subkey("SCHKOLA gGmbH")?.0;
        let sit_client = schkola_ggmbh.create_subkey("S-IT Client")?.0;
        let settings = sit_client.create_subkey("Settings")?.0;
        if Config::get_web_api().is_err() {
            settings.set_value("web_api_https", &"https://127.0.0.1")?;
        }
        sit_client.create_subkey("Client Info")?;
        Ok(())
    }

    pub fn get_web_api() -> Result<String> {
        Self::get_setting("web_api_https")
    }

    pub fn get_uuid() -> Result<Option<Uuid>> {
        let client_info = HKLM
            .open_subkey("SOFTWARE\\SCHKOLA gGmbH\\S-IT Client\\Client Info")?;
        let uuid: Result<String, _> = client_info.get_value("uuid");
        if let Ok(uuid) = uuid {
            Ok(Some(Uuid::parse_str(uuid.as_str())?))
        } else {
            Ok(None)
        }
    }

    pub fn set_uuid(uuid: Uuid) -> Result<()> {
        let client_info = HKLM.open_subkey_with_flags(
            "SOFTWARE\\SCHKOLA gGmbH\\S-IT Client\\Client Info",
            KEY_WRITE,
        )?;
        client_info.set_value("uuid", &uuid.to_string())?;
        Ok(())
    }

    pub fn get_ca_path() -> Result<String> {
        Self::get_setting("ca_path")
    }

    fn get_setting(name: &str) -> Result<String> {
        let s = HKLM.open_subkey("SOFTWARE\\SCHKOLA gGmbH\\S-IT Client\\Settings")?;
        Ok(s.get_value(name)?)
    }
}
