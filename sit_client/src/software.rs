use std::collections::HashMap;

use sit_lib::software::{SoftwareEntry, SoftwareLibrary};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::{RegKey, HKEY};

pub struct Software;

impl Software {
    pub fn get_software_list() -> SoftwareLibrary {
        let mut map = HashMap::new();
        Self::open_sub_key(
            HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            &mut map,
        );
        Self::open_sub_key(
            HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            &mut map,
        );
        Self::open_sub_key(
            HKEY_CURRENT_USER,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            &mut map,
        );
        SoftwareLibrary {
            software: map.into_values().collect(),
        }
    }

    fn open_sub_key(hkey: HKEY, path: &str, map: &mut HashMap<String, SoftwareEntry>) {
        let key = RegKey::predef(hkey).open_subkey(path);
        if let Ok(key) = key {
            Self::extract_software(key, map);
        }
    }

    fn extract_software(uninstall_node: RegKey, map: &mut HashMap<String, SoftwareEntry>) {
        for k in uninstall_node.enum_keys().filter_map(|x| x.ok()) {
            let sub_key = uninstall_node.open_subkey(k);
            if let Ok(sub_key) = sub_key {
                let name: Result<String, _> = sub_key.get_value("DisplayName");
                if let Ok(name) = name {
                    let version: String = sub_key.get_value("DisplayVersion").unwrap_or_default();
                    let software = SoftwareEntry {
                        name: name
                            .trim_end_matches(&version)
                            .trim_end_matches(' ')
                            .to_string(),
                        version,
                        publisher: sub_key.get_value("Publisher").ok(),
                    };
                    map.insert(software.name.clone(), software);
                }
            }
        }
    }
}
