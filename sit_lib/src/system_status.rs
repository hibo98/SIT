use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeList {
    pub volumes: Vec<Volume>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Volume {
    pub drive_letter: String,
    pub label: Option<String>,
    pub file_system: String,
    pub capacity: u64,
    pub free_space: u64,
}
