use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub model: ComputerModel,
    pub memory: PhysicalMemory,
    pub processor: Processor,
    pub disks: Disks,
    pub network: Network,
    pub graphics: GraphicsCard,
    pub bios: BIOS,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComputerModel {
    pub manufacturer: String,
    pub model_family: String,
    pub model: String,
    pub serial_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhysicalMemory {
    pub sticks: Vec<MemoryStick>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryStick {
    pub bank_label: String,
    pub capacity: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Processor {
    pub name: String,
    pub manufacturer: String,
    pub cores: u32,
    pub logical_cores: u32,
    pub clock_speed: u32,
    pub address_width: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disks {
    pub drives: Vec<DiskDrive>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskDrive {
    pub model: String,
    pub serial_number: String,
    pub size: u64,
    pub device_id: String,
    pub status: String,
    pub media_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Network {
    pub adapter: Vec<NetworkAdapter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkAdapter {
    pub name: String,
    pub mac_address: Option<String>,
    pub ip_addresses: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphicsCard {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BIOS {
    pub manufacturer: String,
    pub name: String,
    pub version: String,
}
