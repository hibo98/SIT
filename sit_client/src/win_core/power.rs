use std::ffi::c_void;

use anyhow::Result;
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Devices::DeviceAndDriverInstallation::{
            SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW,
            SetupDiGetDeviceInterfaceDetailA, DIGCF_DEVICEINTERFACE, DIGCF_PRESENT,
            GUID_DEVCLASS_BATTERY, HDEVINFO, SP_DEVICE_INTERFACE_DATA,
            SP_DEVICE_INTERFACE_DETAIL_DATA_A, SP_DEVINFO_DATA,
        },
        Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, HWND},
        Storage::FileSystem::{
            CreateFileA, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_NONE, OPEN_EXISTING,
        },
        System::{
            Power::{
                GetSystemPowerStatus, BATTERY_INFORMATION, BATTERY_MANUFACTURE_DATE,
                BATTERY_QUERY_INFORMATION, BATTERY_QUERY_INFORMATION_LEVEL,
                IOCTL_BATTERY_QUERY_INFORMATION, IOCTL_BATTERY_QUERY_TAG, SYSTEM_POWER_STATUS,
            },
            IO::DeviceIoControl,
        },
    },
};
use sit_lib::hardware::Battery;


pub struct Power;
impl Power {
    pub fn get_system_power_status() -> Result<SYSTEM_POWER_STATUS> {
        let mut system_power_status: SYSTEM_POWER_STATUS = SYSTEM_POWER_STATUS::default();
        unsafe {
            GetSystemPowerStatus(&mut system_power_status)?;
        }
        Ok(system_power_status)
    }

    pub fn get_battery_information() -> Result<Vec<Battery>> {
        let mut return_vec = Vec::new();

        unsafe {
            let h_dev_info = SetupDiGetClassDevsW(
                Some(&GUID_DEVCLASS_BATTERY),
                PCWSTR::null(),
                HWND::default(),
                DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
            )?;

            if h_dev_info.is_invalid() {
                println!("Invalid HANDLE");
            } else {
                let mut device_interface_data = SP_DEVICE_INTERFACE_DATA {
                    cbSize: std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
                    InterfaceClassGuid: GUID_DEVCLASS_BATTERY,
                    Flags: 0,
                    Reserved: 0,
                };
                let mut index = 0;
                while SetupDiEnumDeviceInterfaces(
                    h_dev_info,
                    None,
                    &GUID_DEVCLASS_BATTERY,
                    index,
                    &mut device_interface_data,
                )
                .is_ok()
                {
                    return_vec.push(Power::get_single_bat_info(
                        h_dev_info,
                        &device_interface_data,
                    )?);
                    index += 1;
                }
                SetupDiDestroyDeviceInfoList(h_dev_info)?;
            }
        }

        Ok(return_vec)
    }

    unsafe fn get_single_bat_info(
        h_dev_info: HDEVINFO,
        device_interface_data: &SP_DEVICE_INTERFACE_DATA,
    ) -> Result<Battery> {
        let mut battery_info: BATTERY_INFORMATION = BATTERY_INFORMATION::default();
        //let mut battery_manu_date: BATTERY_MANUFACTURE_DATE = BATTERY_MANUFACTURE_DATE::default();

        let mut requiredsize: u32 = 0;
        let _ = SetupDiGetDeviceInterfaceDetailA(
            h_dev_info,
            device_interface_data,
            None,
            0,
            Some(&mut requiredsize),
            None,
        );

        let mut p_interface_detail_data: Vec<u8> = vec![0; requiredsize as usize];
        let p_interface_detail_data =
            p_interface_detail_data.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_A;
        (*p_interface_detail_data).cbSize =
            std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_A>() as u32;
        let mut dev_info_data = SP_DEVINFO_DATA {
            cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ClassGuid: GUID_DEVCLASS_BATTERY,
            DevInst: 0,
            Reserved: 0,
        };

        SetupDiGetDeviceInterfaceDetailA(
            h_dev_info,
            device_interface_data,
            Some(p_interface_detail_data),
            requiredsize,
            None,
            Some(&mut dev_info_data),
        )?;
        let device_path =
            PCSTR::from_raw((*p_interface_detail_data).DevicePath.as_ptr() as *const u8);

        let handle = CreateFileA(
            device_path,
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            HANDLE::default(),
        )?;

        let mut battery_tag: u32 = 0;
        let mut returned_bytes: u32 = 0;

        DeviceIoControl(
            handle,
            IOCTL_BATTERY_QUERY_TAG,
            None,
            0,
            Some(&mut battery_tag as *mut _ as *mut c_void),
            std::mem::size_of::<u32>() as u32,
            Some(&mut returned_bytes),
            None,
        )?;

        let query_info = BATTERY_QUERY_INFORMATION {
            BatteryTag: battery_tag,
            InformationLevel: BATTERY_QUERY_INFORMATION_LEVEL(0),
            AtRate: 0,
        };

        DeviceIoControl(
            handle,
            IOCTL_BATTERY_QUERY_INFORMATION,
            Some(&query_info as *const _ as *const c_void),
            std::mem::size_of::<BATTERY_QUERY_INFORMATION>() as u32,
            Some(&mut battery_info as *mut _ as *mut c_void),
            std::mem::size_of::<BATTERY_INFORMATION>() as u32,
            Some(&mut returned_bytes),
            None,
        )?;

        // let query_info = BATTERY_QUERY_INFORMATION {
        //     BatteryTag: battery_tag,
        //     InformationLevel: BATTERY_QUERY_INFORMATION_LEVEL(5),
        //     AtRate: 0,
        // };

        // DeviceIoControl(
        //     handle,
        //     IOCTL_BATTERY_QUERY_INFORMATION,
        //     Some(&query_info as *const _ as *const c_void),
        //     std::mem::size_of::<BATTERY_QUERY_INFORMATION>() as u32,
        //     Some(&mut battery_manu_date as *mut _ as *mut c_void),
        //     std::mem::size_of::<BATTERY_MANUFACTURE_DATE>() as u32,
        //     Some(&mut returned_bytes),
        //     None,
        // )?;

        let mut bat_manu_name = vec![0u16; 256];

        let query_info = BATTERY_QUERY_INFORMATION {
            BatteryTag: battery_tag,
            InformationLevel: BATTERY_QUERY_INFORMATION_LEVEL(6),
            AtRate: 0,
        };

        DeviceIoControl(
            handle,
            IOCTL_BATTERY_QUERY_INFORMATION,
            Some(&query_info as *const _ as *const c_void),
            std::mem::size_of::<BATTERY_QUERY_INFORMATION>() as u32,
            Some(bat_manu_name.as_mut_ptr() as *mut c_void),
            (bat_manu_name.len() * 2) as u32,
            Some(&mut returned_bytes),
            None,
        )?;

        let mut bat_uid = vec![0u16; 256];

        let query_info = BATTERY_QUERY_INFORMATION {
            BatteryTag: battery_tag,
            InformationLevel: BATTERY_QUERY_INFORMATION_LEVEL(7),
            AtRate: 0,
        };

        DeviceIoControl(
            handle,
            IOCTL_BATTERY_QUERY_INFORMATION,
            Some(&query_info as *const _ as *const c_void),
            std::mem::size_of::<BATTERY_QUERY_INFORMATION>() as u32,
            Some(bat_uid.as_mut_ptr() as *mut c_void),
            (bat_uid.len() * 2) as u32,
            Some(&mut returned_bytes),
            None,
        )?;

        let mut bat_serial_number = vec![0u16; 256];

        let query_info = BATTERY_QUERY_INFORMATION {
            BatteryTag: battery_tag,
            InformationLevel: BATTERY_QUERY_INFORMATION_LEVEL(8),
            AtRate: 0,
        };

        DeviceIoControl(
            handle,
            IOCTL_BATTERY_QUERY_INFORMATION,
            Some(&query_info as *const _ as *const c_void),
            std::mem::size_of::<BATTERY_QUERY_INFORMATION>() as u32,
            Some(bat_serial_number.as_mut_ptr() as *mut c_void),
            (bat_serial_number.len() * 2) as u32,
            Some(&mut returned_bytes),
            None,
        )?;

        let bat_info = Battery {
            id: PCWSTR::from_raw(bat_uid.as_ptr()).to_string()?,
            chemistry: PCSTR::from_raw(battery_info.Chemistry.as_ptr()).to_string()?,
            cycle_count: battery_info.CycleCount,
            designed_capacity: battery_info.DesignedCapacity,
            full_charged_capacity: battery_info.FullChargedCapacity,
            manufacturer: PCWSTR::from_raw(bat_manu_name.as_ptr()).to_string()?,
            serial_number: PCWSTR::from_raw(bat_serial_number.as_ptr()).to_string()?,
        };

        CloseHandle(handle)?;

        Ok(bat_info)
    }
}
