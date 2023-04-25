use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use uuid::Uuid;

use crate::{database::Database, display_util, ms_magic};

#[derive(Clone, Debug, Serialize)]
struct Profile {
    pub user_sid: String,
    pub user_name: String,
    pub health_status: String,
    pub roaming_configured: bool,
    pub roaming_path: Option<String>,
    pub roaming_preference: Option<bool>,
    pub last_use_time: String,
    pub last_download_time: String,
    pub last_upload_time: String,
    pub status: Vec<String>,
    pub size: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Memory {
    pub capacity: String,
    pub stick_count: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct MemoryStick {
    pub capacity: String,
    pub bank_label: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Disk {
    pub model: String,
    pub serial_number: String,
    pub size: String,
    pub device_id: String,
    pub status: String,
    pub media_type: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct VolumeStatus {
    pub drive_letter: String,
    pub label: Option<String>,
    pub file_system: String,
    pub capacity: String,
    pub free_space: String,
    pub occupied_space: String,
    pub occupied_percentage: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserProfilePaths {
    pub path: String,
    pub size: String,
}

#[get("/")]
pub fn index(database: &State<Database>) -> Template {
    let client_info = database.get_clients_with_os_info().unwrap_or(vec![]);
    Template::render("clients", context! { clients: client_info })
}

#[get("/<uuid>")]
pub fn client(database: &State<Database>, uuid: Uuid) -> Template {
    let client = database.get_client(&uuid);
    let os_info = database.get_client_os_info(&uuid);
    if let (Ok(client), Ok(os_info)) = (client, os_info) {
        Template::render("client", context! { client, os_info })
    } else {
        Template::render("client", context! {})
    }
}

#[get("/<uuid>/profiles")]
pub fn profiles(database: &State<Database>, uuid: Uuid) -> Template {
    let client = database.get_client(&uuid);
    let os_info = database.get_client_os_info(&uuid);
    let client_profiles = database.get_client_profiles(&uuid);
    if let (Ok(client), Ok(os_info), Ok(client_profiles)) = (client, os_info, client_profiles) {
        let profiles: Vec<Profile> = client_profiles
            .into_iter()
            .map(|(up, u)| Profile {
                user_sid: u.sid,
                user_name: u.username.unwrap_or("<unknown user>".to_owned()),
                health_status: ms_magic::resolve_profile_health_status(up.health_status),
                roaming_configured: up.roaming_configured,
                roaming_path: up.roaming_path,
                roaming_preference: up.roaming_preference,
                last_use_time: display_util::format_date_time(up.last_use_time),
                last_download_time: up
                    .last_download_time
                    .map(display_util::format_date_time)
                    .unwrap_or_default(),
                last_upload_time: up
                    .last_upload_time
                    .map(display_util::format_date_time)
                    .unwrap_or_default(),
                status: ms_magic::resolve_profile_status(up.status),
                size: display_util::format_option_big_decimal(
                    &up.size,
                    display_util::format_filesize_byte,
                ),
            })
            .collect();
        Template::render("clients/profiles", context! { profiles, client, os_info, uuid: uuid.to_string() })
    } else {
        Template::render("clients/profiles", context! {})
    }
}

#[get("/<uuid>/profiles/<sid>")]
pub fn profile_paths(database: &State<Database>, uuid: Uuid, sid: String) -> Template {
    let client = database.get_client(&uuid);
    let os_info = database.get_client_os_info(&uuid);
    let user = database.get_user(&sid);
    let profile_paths = database.get_profile_paths(&uuid, &sid);
    if let (Ok(client), Ok(os_info), Ok(user), Ok(profile_paths)) =
        (client, os_info, user, profile_paths)
    {
        let paths: Vec<UserProfilePaths> = profile_paths
            .into_iter()
            .map(|p| UserProfilePaths {
                path: p.path,
                size: display_util::format_big_decimal(&p.size, display_util::format_filesize_byte),
            })
            .collect();
        Template::render(
            "clients/profiles_path",
            context! { paths, user, client, os_info },
        )
    } else {
        Template::render("clients/profiles_path", context! {})
    }
}

#[get("/<uuid>/software")]
pub fn software(database: &State<Database>, uuid: Uuid) -> Template {
    let client = database.get_client(&uuid);
    let os_info = database.get_client_os_info(&uuid);
    let software = database.get_client_software(uuid);
    if let (Ok(client), Ok(os_info), Ok(software)) = (client, os_info, software) {
        Template::render("clients/software", context! { software, client, os_info })
    } else {
        Template::render("clients/software", context! {})
    }
}

#[get("/<uuid>/hardware")]
pub fn hardware(database: &State<Database>, uuid: Uuid) -> Template {
    let client = database.get_client(&uuid);
    let os_info = database.get_client_os_info(&uuid);
    if let (Ok(client), Ok(os_info)) = (client, os_info) {
        let processors = database.get_client_processors(uuid).unwrap_or(vec![]);
        let memory: Vec<Memory> = database
            .get_client_memory(uuid)
            .unwrap_or(vec![])
            .into_iter()
            .map(|m| Memory {
                capacity: display_util::format_option_big_decimal(
                    &m.capacity,
                    display_util::format_filesize_byte_iec,
                ),
                stick_count: m.stick_count,
            })
            .collect();
        let memory_sticks: Vec<MemoryStick> = database
            .get_client_memory_sticks(uuid)
            .unwrap_or(vec![])
            .into_iter()
            .map(|m| MemoryStick {
                capacity: display_util::format_option_big_decimal(
                    &m.capacity,
                    display_util::format_filesize_byte_iec,
                ),
                bank_label: m.bank_label,
            })
            .collect();
        let graphics_cards = database.get_client_graphics_cards(uuid).unwrap_or(vec![]);
        let disks: Vec<Disk> = database
            .get_client_disks(uuid)
            .unwrap_or(vec![])
            .into_iter()
            .map(|d| Disk {
                model: d.model,
                serial_number: d.serial_number,
                size: display_util::format_option_big_decimal(
                    &d.size,
                    display_util::format_filesize_byte,
                ),
                device_id: d.device_id,
                status: d.status,
                media_type: d.media_type,
            })
            .collect();
        let computer_models = database.get_client_computer_model(uuid).unwrap_or(vec![]);
        let bios_list = database.get_client_bios(uuid).unwrap_or(vec![]);
        let network_adapters = database.get_client_network_adapters(uuid).unwrap_or(vec![]);
        Template::render(
            "clients/hardware",
            context! { processors, memory, memory_sticks, graphics_cards, disks, computer_models, bios_list, network_adapters, client, os_info },
        )
    } else {
        Template::render("clients/hardware", context! {})
    }
}

#[get("/<uuid>/status")]
pub fn status(database: &State<Database>, uuid: Uuid) -> Template {
    let client = database.get_client(&uuid);
    let os_info = database.get_client_os_info(&uuid);
    if let (Ok(client), Ok(os_info)) = (client, os_info) {
        let volumes: Vec<VolumeStatus> = database
            .get_client_volume_status(uuid)
            .unwrap_or(vec![])
            .into_iter()
            .map(|v| VolumeStatus {
                drive_letter: v.drive_letter,
                label: v.label,
                file_system: v.file_system,
                capacity: display_util::format_big_decimal(
                    &v.capacity,
                    display_util::format_filesize_byte,
                ),
                free_space: display_util::format_big_decimal(
                    &v.free_space,
                    display_util::format_filesize_byte,
                ),
                occupied_space: display_util::format_big_decimal(
                    &(&v.capacity - &v.free_space),
                    display_util::format_filesize_byte,
                ),
                occupied_percentage: display_util::format_bd_percentage(
                    &(&v.capacity - &v.free_space),
                    &v.capacity,
                ),
            })
            .collect();
        Template::render("clients/status", context! { volumes, client, os_info })
    } else {
        Template::render("clients/status", context! {})
    }
}

#[get("/<uuid>/licenses")]
pub fn licenses(database: &State<Database>, uuid: Uuid) -> Template {
    let client = database.get_client(&uuid);
    let os_info = database.get_client_os_info(&uuid);
    if let (Ok(client), Ok(os_info)) = (client, os_info) {
        let licenses = database.get_client_licenses(uuid).unwrap_or(vec![]);
        Template::render("clients/licenses", context! { licenses, client, os_info })
    } else {
        Template::render("clients/licenses", context! {})
    }
}
