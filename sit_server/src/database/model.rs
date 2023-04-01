use uuid::Uuid;

use super::schema::*;
use bigdecimal::BigDecimal;
use chrono::naive::NaiveDateTime;
use rocket::serde::Serialize;

#[derive(Debug, Queryable, Serialize, Clone)]
pub struct Client {
    pub id: i32,
    pub uuid: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = client)]
pub struct NewClient<'a> {
    pub uuid: &'a Uuid,
}

#[derive(Debug, Queryable, Serialize, Clone)]
pub struct OsInfo {
    pub client_id: i32,
    pub os: Option<String>,
    pub os_version: Option<String>,
    pub computer_name: String,
    pub domain: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = os_info)]
pub struct NewOsInfo<'a> {
    pub client_id: &'a i32,
    pub computer_name: &'a str,
}

#[derive(AsChangeset)]
#[diesel(table_name = os_info)]
pub struct UpdateOsInfo<'a> {
    pub os: Option<&'a String>,
    pub os_version: Option<&'a String>,
    pub computer_name: Option<&'a String>,
    pub domain: Option<&'a String>,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct SoftwareInfo {
    pub id: i32,
    pub name: String,
    pub publisher: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = software_info)]
pub struct NewSoftwareInfo<'a> {
    pub name: &'a String,
    pub publisher: Option<&'a String>,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct SoftwareVersion {
    pub id: i32,
    pub software_id: i32,
    pub version: String,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct SoftwareVersionWithCount {
    pub id: i32,
    pub software_id: i32,
    pub version: String,
    pub count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = software_version)]
pub struct NewSoftwareVersion<'a> {
    pub software_id: &'a i32,
    pub version: &'a String,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct SoftwareList {
    pub client_id: i32,
    pub software_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = software_list)]
pub struct NewSoftwareList<'a> {
    pub client_id: &'a i32,
    pub software_id: &'a i32,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct User {
    pub id: i32,
    pub sid: String,
    pub username: Option<String>,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct UserWithProfileCount {
    pub id: i32,
    pub sid: String,
    pub username: Option<String>,
    pub count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = user)]
pub struct NewUser<'a> {
    pub sid: &'a String,
    pub username: Option<&'a String>,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct UserProfile {
    pub client_id: i32,
    pub user_id: i32,
    pub health_status: i16,
    pub roaming_configured: bool,
    pub roaming_path: Option<String>,
    pub roaming_preference: Option<bool>,
    pub last_use_time: NaiveDateTime,
    pub last_download_time: Option<NaiveDateTime>,
    pub last_upload_time: Option<NaiveDateTime>,
    pub status: i64,
    pub size: Option<BigDecimal>,
}

#[derive(Insertable)]
#[diesel(table_name = userprofile)]
pub struct NewUserProfileWithSize<'a> {
    pub client_id: &'a i32,
    pub user_id: &'a i32,
    pub health_status: &'a i16,
    pub roaming_configured: &'a bool,
    pub roaming_path: Option<&'a String>,
    pub roaming_preference: Option<&'a bool>,
    pub last_use_time: &'a NaiveDateTime,
    pub last_download_time: Option<NaiveDateTime>,
    pub last_upload_time: Option<NaiveDateTime>,
    pub status: &'a i64,
    pub size: Option<BigDecimal>,
}

#[derive(Insertable)]
#[diesel(table_name = userprofile)]
pub struct NewUserProfileWithoutSize<'a> {
    pub client_id: &'a i32,
    pub user_id: &'a i32,
    pub health_status: &'a i16,
    pub roaming_configured: &'a bool,
    pub roaming_path: Option<&'a String>,
    pub roaming_preference: Option<&'a bool>,
    pub last_use_time: &'a NaiveDateTime,
    pub last_download_time: Option<NaiveDateTime>,
    pub last_upload_time: Option<NaiveDateTime>,
    pub status: &'a i64,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct ComputerModel {
    pub client_id: i32,
    pub manufacturer:String,
    pub model_family: String,
    pub serial_number: String,
}

#[derive(Insertable)]
#[diesel(table_name = computer_model)]
pub struct NewComputerModel<'a> {
    pub client_id: &'a i32,
    pub manufacturer: &'a String,
    pub model_family: &'a String,
    pub serial_number: &'a String,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct Memory {
    pub client_id: i32,
    pub capacity: Option<BigDecimal>,
    pub stick_count: i64,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct MemoryStick {
    pub id: i32,
    pub client_id: i32,
    pub capacity: Option<BigDecimal>,
    pub bank_label: String,
}

#[derive(Insertable)]
#[diesel(table_name = memory_stick)]
pub struct NewMemoryStick<'a> {
    pub client_id: &'a i32,
    pub capacity: &'a BigDecimal,
    pub bank_label: &'a String,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct Processor {
    pub client_id: i32,
    pub name: String,
    pub manufacturer: String,
    pub cores: i64,
    pub logical_cores: i64,
    pub clock_speed: i64,
    pub address_width: i32,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct ProcessorCount {
    pub name: String,
    pub manufacturer: String,
    pub cores: Option<i64>,
    pub logical_cores: Option<i64>,
    pub clock_speed: Option<i64>,
    pub address_width: Option<i32>,
    pub count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = processor)]
pub struct NewProcessor<'a> {
    pub client_id: &'a i32,
    pub name: &'a String,
    pub manufacturer: &'a String,
    pub cores: &'a i64,
    pub logical_cores: &'a i64,
    pub clock_speed: &'a i64,
    pub address_width: &'a i32,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct Disk {
    pub id: i32,
    pub client_id: i32,
    pub model: String,
    pub serial_number: String,
    pub size: Option<BigDecimal>,
    pub device_id: String,
    pub status: String,
    pub media_type: String,
}

#[derive(Insertable)]
#[diesel(table_name = disks)]
pub struct NewDisk<'a> {
    pub client_id: &'a i32,
    pub model: &'a String,
    pub serial_number: &'a String,
    pub size: Option<BigDecimal>,
    pub device_id: &'a String,
    pub status: &'a String,
    pub media_type: &'a String,
}

#[derive(Debug, Queryable, Serialize)]
pub struct NetworkAdapter {
    pub id: i32,
    pub client_id: i32,
    pub name: String,
    pub mac_address: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = network_adapter)]
pub struct NewNetworkAdapter<'a> {
    pub client_id: &'a i32,
    pub name: &'a String,
    pub mac_address: Option<&'a String>,
}

#[derive(Insertable)]
#[diesel(table_name = network_adapter_ip)]
pub struct NewNetworkAdapterIp<'a> {
    pub adapter_id: &'a i32,
    pub ip: &'a String,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct GraphicsCard {
    pub client_id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = graphics_card)]
pub struct NewGraphicsCard<'a> {
    pub client_id: &'a i32,
    pub name: &'a String,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct Bios {
    pub client_id: i32,
    pub name: String,
    pub manufacturer: String,
    pub version: String,
}

#[derive(Insertable)]
#[diesel(table_name = bios)]
pub struct NewBios<'a> {
    pub client_id: &'a i32,
    pub name: &'a String,
    pub manufacturer: &'a String,
    pub version: &'a String,
}
