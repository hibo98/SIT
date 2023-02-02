use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use uuid::Uuid;

use crate::{database::Database, display_util};

#[derive(Clone, Debug, Serialize)]
struct Profile {
    pub user_sid: String,
    pub user_name: String,
    pub health_status: i16,
    pub roaming_configured: bool,
    pub roaming_path: Option<String>,
    pub roaming_preference: Option<bool>,
    pub last_use_time: String,
    pub last_download_time: String,
    pub last_upload_time: String,
    pub status: i64,
    pub size: String,
}

#[get("/")]
pub fn index(database: &State<Database>) -> Template {
    let client_info = database.get_clients_info().unwrap_or(vec![]);
    Template::render("clients", context! { clients: client_info })
}

#[get("/<uuid>")]
pub fn client(database: &State<Database>, uuid: Uuid) -> Template {
    let client_info = database.get_client_info(uuid);
    if let Ok(client_info) = client_info {
        Template::render("client", context! { client: client_info })
    } else {
        Template::render("client", context! {})
    }
}

#[get("/<uuid>/profiles")]
pub fn profiles(database: &State<Database>, uuid: Uuid) -> Template {
    let client_profiles = database.get_client_profiles(uuid);
    if let Ok(client_profiles) = client_profiles {
        let profiles: Vec<Profile> = client_profiles
            .iter()
            .map(|(up, u)| Profile {
                user_sid: u.sid.clone(),
                user_name: u.username.clone().unwrap_or("<no username>".to_string()),
                health_status: up.health_status,
                roaming_configured: up.roaming_configured,
                roaming_path: up.roaming_path.clone(),
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
                status: up.status,
                size: up
                    .size
                    .as_ref()
                    .map(|size| display_util::format_filesize_byte(size.digits() as f64, 0))
                    .unwrap_or_default(),
            })
            .collect();
        Template::render("client_profiles", context! { profiles })
    } else {
        Template::render("client_profiles", context! {})
    }
}

#[get("/<uuid>/software")]
pub fn software(database: &State<Database>, uuid: Uuid) -> Template {
    let client_software = database.get_client_software(uuid);
    if let Ok(client_software) = client_software {
        Template::render("client_software", context! { software: client_software })
    } else {
        Template::render("client_software", context! {})
    }
}
