use bigdecimal::ToPrimitive;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use uuid::Uuid;

use crate::{database::Database, display_util};

#[derive(Clone, Debug, Serialize)]
struct Profile {
    pub client_uuid: Uuid,
    pub os_computer_name: String,
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
    let profiles = database.get_profiles().unwrap_or(vec![]);
    Template::render("profiles", context! { profiles })
}

#[get("/<sid>")]
pub fn profile(database: &State<Database>, sid: String) -> Template {
    let profiles_result = database.get_profile_info(sid);
    if let Ok(profiles) = profiles_result {
        let profile: Vec<Profile> = profiles
            .iter()
            .map(|(up, _, c, os)| Profile {
                client_uuid: c.uuid,
                os_computer_name: os
                    .as_ref()
                    .map(|os| {
                        if let Some(domain) = &os.domain {
                            format!("{}.{}", os.computer_name, domain)
                        } else {
                            os.computer_name.clone()
                        }
                    })
                    .unwrap_or("<no computer name>".to_string()),
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
                    .map(|size| {
                        size.to_f64()
                            .map(|size| display_util::format_filesize_byte(size, 0))
                            .unwrap_or_default()
                    })
                    .unwrap_or_default(),
            })
            .collect();
        Template::render("profile", context! { profile })
    } else {
        Template::render("profile", context! {})
    }
}
