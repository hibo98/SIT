use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Register {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,
}
