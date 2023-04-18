#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use anyhow::{bail, Result};
use serde::Deserialize;
use sit_lib::hardware::*;
use wmi::WMIConnection;

pub struct Hardware;

#[derive(Deserialize, Debug)]
struct Win32_ComputerSystem {
    SystemFamily: String,
    Model: String,
    Manufacturer: String,
}

#[derive(Deserialize, Debug)]
struct Win32_PhysicalMemory {
    BankLabel: String,
    Capacity: u64,
}

#[derive(Deserialize, Debug)]
struct Win32_Processor {
    Name: String,
    NumberOfCores: u32,
    NumberOfLogicalProcessors: u32,
    MaxClockSpeed: u32,
    AddressWidth: u16,
    Manufacturer: String,
}

#[derive(Deserialize, Debug)]
struct Win32_DiskDrive {
    Model: String,
    SerialNumber: String,
    Size: u64,
    DeviceID: String,
    Status: String,
    MediaType: String,
}

#[derive(Deserialize, Debug)]
struct Win32_NetworkAdapter {
    Name: String,
    MACAddress: Option<String>,
    PhysicalAdapter: bool,
}

#[derive(Deserialize, Debug)]
struct Win32_NetworkAdapterConfiguration {
    Description: String,
    IPAddress: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Win32_VideoController {
    Name: String,
}

#[derive(Deserialize, Debug)]
struct Win32_SystemEnclosure {
    SerialNumber: String,
}

#[derive(Deserialize, Debug)]
struct Win32_BIOS {
    Manufacturer: String,
    Name: String,
    Version: String,
}

impl Hardware {
    pub fn get_hardware_info(wmi_con: &WMIConnection) -> Result<HardwareInfo> {
        let model = Self::get_model(wmi_con)?;
        let memory = Self::get_memory(wmi_con)?;
        let processor = Self::get_processor(wmi_con)?;
        let disks = Self::get_disks(wmi_con)?;
        let network = Self::get_network(wmi_con)?;
        let graphics = Self::get_graphics(wmi_con)?;
        let bios = Self::get_bios(wmi_con)?;

        Ok(HardwareInfo {
            model,
            memory,
            processor,
            disks,
            network,
            graphics,
            bios,
        })
    }

    fn get_model(wmi_con: &WMIConnection) -> Result<ComputerModel> {
        let wmi_cs: Vec<Win32_ComputerSystem> = wmi_con.query()?;
        let wmi_se: Vec<Win32_SystemEnclosure> = wmi_con.query()?;

        if let (Some(wmi_cs), Some(wmi_se)) = (wmi_cs.last(), wmi_se.last()) {
            Ok(ComputerModel {
                manufacturer: wmi_cs.Manufacturer.clone(),
                model_family: wmi_cs.SystemFamily.clone(),
                model: wmi_cs.Model.clone(),
                serial_number: wmi_se.SerialNumber.clone(),
            })
        } else {
            bail!("Empty result on computer model");
        }
    }

    fn get_memory(wmi_con: &WMIConnection) -> Result<PhysicalMemory> {
        let mut sticks = Vec::new();
        let wmi_pm: Vec<Win32_PhysicalMemory> = wmi_con.query()?;
        for pm in wmi_pm {
            sticks.push(MemoryStick {
                bank_label: pm.BankLabel,
                capacity: pm.Capacity,
            })
        }
        Ok(PhysicalMemory { sticks })
    }

    fn get_processor(wmi_con: &WMIConnection) -> Result<Processor> {
        let wmi_cpu: Vec<Win32_Processor> = wmi_con.query()?;
        if let Some(wmi_cpu) = wmi_cpu.last() {
            Ok(Processor {
                name: wmi_cpu.Name.trim().to_string(),
                manufacturer: wmi_cpu.Manufacturer.clone(),
                cores: wmi_cpu.NumberOfCores,
                logical_cores: wmi_cpu.NumberOfLogicalProcessors,
                clock_speed: wmi_cpu.MaxClockSpeed,
                address_width: wmi_cpu.AddressWidth,
            })
        } else {
            bail!("Empty result on processor");
        }
    }

    fn get_disks(wmi_con: &WMIConnection) -> Result<Disks> {
        let mut drives = Vec::new();
        let wmi_dd: Vec<Win32_DiskDrive> = wmi_con.query()?;
        for dd in wmi_dd {
            drives.push(DiskDrive {
                model: dd.Model,
                serial_number: dd.SerialNumber.trim().to_string(),
                size: dd.Size,
                device_id: dd.DeviceID,
                status: dd.Status,
                media_type: dd.MediaType,
            })
        }
        Ok(Disks { drives })
    }

    fn get_network(wmi_con: &WMIConnection) -> Result<Network> {
        let mut adapter = Vec::new();
        let wmi_na: Vec<Win32_NetworkAdapter> = wmi_con.query()?;
        let wmi_nac: Vec<Win32_NetworkAdapterConfiguration> = wmi_con.query()?;
        for na in wmi_na {
            if na.PhysicalAdapter {
                let nac = Hardware::get_net_config(&wmi_nac, &na.Name);
                adapter.push(NetworkAdapter {
                    name: na.Name,
                    mac_address: na.MACAddress,
                    ip_addresses: nac.map(|n| n.IPAddress.clone()),
                })
            }
        }
        Ok(Network { adapter })
    }

    fn get_graphics(wmi_con: &WMIConnection) -> Result<GraphicsCard> {
        let wmi_vc: Vec<Win32_VideoController> = wmi_con.query()?;
        if let Some(wmi_vc) = wmi_vc.last() {
            Ok(GraphicsCard {
                name: wmi_vc.Name.clone(),
            })
        } else {
            bail!("Empty result on graphics");
        }
    }

    fn get_bios(wmi_con: &WMIConnection) -> Result<BIOS> {
        let wmi_bios: Vec<Win32_BIOS> = wmi_con.query()?;
        if let Some(wmi_bios) = wmi_bios.last() {
            Ok(BIOS {
                name: wmi_bios.Name.clone(),
                manufacturer: wmi_bios.Manufacturer.clone(),
                version: wmi_bios.Version.clone(),
            })
        } else {
            bail!("Empty result on bios");
        }
    }

    fn get_net_config<'a>(
        wmi_nac: &'a Vec<Win32_NetworkAdapterConfiguration>,
        name: &str,
    ) -> Option<&'a Win32_NetworkAdapterConfiguration> {
        for nac in wmi_nac {
            if nac.Description == name {
                return Some(nac.to_owned());
            }
        }
        None
    }
}
