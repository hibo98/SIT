#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


use serde::Deserialize;
use sit_lib::hardware::{BIOS, ComputerModel, DiskDrive, Disks, GraphicsCard, HardwareInfo, MemoryStick, Network, NetworkAdapter, PhysicalMemory, Processor};
use wmi::{WMIConnection, WMIError};

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
    pub fn get_hardware_info(wmi_con: &WMIConnection) -> Result<HardwareInfo, WMIError> {
        let wmi_cs: Vec<Win32_ComputerSystem> = wmi_con.query()?;
        let wmi_se: Vec<Win32_SystemEnclosure> = wmi_con.query()?;
        let model = if let Some(wmi_cs) = wmi_cs.last() {
            wmi_se.last().map(|wmi_se| ComputerModel {
                manufacturer: wmi_cs.Manufacturer.clone(),
                model_family: wmi_cs.SystemFamily.clone(),
                model: wmi_cs.Model.clone(),
                serial_number: wmi_se.SerialNumber.clone(),
            })
        } else { None };

        let mut sticks = Vec::new();
        let wmi_pm: Vec<Win32_PhysicalMemory> = wmi_con.query()?;
        for pm in wmi_pm {
            sticks.push(MemoryStick {
                bank_label: pm.BankLabel,
                capacity: pm.Capacity,
            })
        }
        let memory = PhysicalMemory {
            sticks,
        };

        let wmi_cpu: Vec<Win32_Processor> = wmi_con.query()?;
        let processor = wmi_cpu.last().map(|wmi_cpu| Processor {
            name: wmi_cpu.Name.trim().to_string(),
            manufacturer: wmi_cpu.Manufacturer.clone(),
            cores: wmi_cpu.NumberOfCores,
            logical_cores: wmi_cpu.NumberOfLogicalProcessors,
            clock_speed: wmi_cpu.MaxClockSpeed,
            address_width: wmi_cpu.AddressWidth,
        });

        let mut drives = Vec::new();
        let wmi_dd: Vec<Win32_DiskDrive> = wmi_con.query()?;
        for dd in wmi_dd {
            drives.push(DiskDrive {
                model: dd.Model.clone(),
                serial_number: dd.SerialNumber.trim().to_string(),
                size: dd.Size,
                device_id: dd.DeviceID.clone(),
                status: dd.Status.clone(),
                media_type: dd.MediaType.clone(),
            })
        }
        let disks = Disks {
            drives,
        };

        let mut adapter = Vec::new();
        let wmi_na: Vec<Win32_NetworkAdapter> = wmi_con.query()?;
        let wmi_nac: Vec<Win32_NetworkAdapterConfiguration> = wmi_con.query()?;
        for na in wmi_na {
            if na.PhysicalAdapter {
                let nac = Hardware::get_net_config(&wmi_nac, &na.Name);
                adapter.push(NetworkAdapter {
                    name: na.Name.clone(),
                    mac_address: na.MACAddress,
                    ip_addresses: nac.map(|n| n.IPAddress.clone()),
                })
            }
        }
        let network = Network {
            adapter,
        };

        let wmi_vc: Vec<Win32_VideoController> = wmi_con.query()?;
        let graphics = wmi_vc.last().map(|wmi_vc| GraphicsCard {
            name: wmi_vc.Name.clone(),
        });

        let wmi_bios: Vec<Win32_BIOS> = wmi_con.query()?;
        let bios = wmi_bios.last().map(|wmi_bios| BIOS {
            name: wmi_bios.Name.clone(),
            manufacturer: wmi_bios.Manufacturer.clone(),
            version: wmi_bios.Version.clone(),
        });
        if let Some(model) = model {
            if let Some(processor) = processor {
                if let Some(graphics) = graphics {
                    if let Some(bios) = bios {
                        return Ok(HardwareInfo {
                            model,
                            memory,
                            processor,
                            disks,
                            network,
                            graphics,
                            bios,
                        });
                    }
                }
            }
        }
        Err(WMIError::ResultEmpty)
    }

    fn get_net_config<'a>(wmi_nac: &'a Vec<Win32_NetworkAdapterConfiguration>, name: &str) -> Option<&'a Win32_NetworkAdapterConfiguration> {
        for nac in wmi_nac {
            if nac.Description == name {
                return Some(nac.to_owned());
            }
        }
        None
    }
}
