use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct SoftwareLibrary {
    pub software: Vec<SoftwareEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SoftwareEntry {
    pub name: String,
    pub version: String,
    pub publisher: Option<String>,
}
