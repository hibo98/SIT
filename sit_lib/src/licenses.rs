use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseBundle {
    pub licenses: Vec<License>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    pub key: String,
}
