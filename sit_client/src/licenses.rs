use anyhow::Result;
use sit_lib::licenses::{License, LicenseBundle};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

pub struct Licenses;
impl Licenses {
    pub fn collect_licenses() -> Result<LicenseBundle> {
        let licenses: Vec<License> = vec![Self::get_windows_key()?];
        Ok(LicenseBundle { licenses })
    }

    pub fn get_windows_key() -> Result<License> {
        let mut key_output = String::new();
        let chars = vec![
            "B", "C", "D", "F", "G", "H", "J", "K", "M", "P", "Q", "R", "T", "V", "W", "X", "Y",
            "2", "3", "4", "6", "7", "8", "9",
        ];
        let key_offset = 52;

        let win_curr_ver = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")?;
        let name: String = win_curr_ver.get_value("ProductName")?;
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

        Ok(License {
            name,
            key: key_output,
        })
    }
}
