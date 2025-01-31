use rocket::{response::Redirect, Route, State};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use uuid::Uuid;

use crate::{auth::User, database::Database};

use super::{display_util, ms_magic};

#[derive(Clone, Debug, Serialize)]
struct Profile {
    pub client_uuid: Uuid,
    pub os_computer_name: String,
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
struct UserWithProfileCount {
    pub id: i32,
    pub sid: String,
    pub username: String,
    pub domain: String,
    pub count: i64,
}

#[get("/")]
fn index(database: &State<Database>, user: User) -> Template {
    let profiles: Vec<UserWithProfileCount> = database
        .user_manager()
        .get_profiles()
        .unwrap_or_default()
        .into_iter()
        .map(|p| UserWithProfileCount {
            id: p.id,
            sid: p.sid,
            username: p.username.unwrap_or("<_user>".to_owned()),
            domain: p.domain.unwrap_or("<_domain>".to_owned()),
            count: p.count,
        })
        .collect();
    Template::render("profile/index", context! { profiles, user })
}

#[get("/<sid>")]
fn profile(database: &State<Database>, sid: String, user: User) -> Template {
    let user_id = database.user_manager().get_user_id_for_sid(&sid);
    if let Ok(Some(user_id)) = user_id {
        let profiles_result = database.user_manager().get_profile_info(user_id);
        if let Ok(profiles) = profiles_result {
            let profile: Vec<Profile> = profiles
                .into_iter()
                .map(|(up, c, os)| Profile {
                    client_uuid: c.uuid,
                    os_computer_name: os
                        .map_or("<_computer_name>".to_string(), |os| {
                            if let Some(domain) = os.domain {
                                format!("{}.{}", os.computer_name, domain)
                            } else {
                                os.computer_name
                            }
                        }),
                    health_status: ms_magic::resolve_profile_health_status(up.health_status),
                    roaming_configured: up.roaming_configured,
                    roaming_path: up.roaming_path,
                    roaming_preference: up.roaming_preference,
                    last_use_time: up
                        .last_use_time
                        .map(display_util::format_date_time)
                        .unwrap_or_default(),
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
            Template::render("profile/profile", context! { profile, user })
        } else {
            Template::render("profile/profile", context! {})
        }
    } else {
        Template::render("profile/profile", context! {})
    }
}

#[get("/<_..>", rank = 10)]
fn catch_all() -> Redirect {
    Redirect::to(uri!("/auth/login"))
}

pub fn routes() -> Vec<Route> {
    routes![index, profile, catch_all]
}
