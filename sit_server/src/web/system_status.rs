use rocket::{Route, State};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use uuid::Uuid;

use crate::database::Database;

use super::display_util;

#[derive(Clone, Debug, Serialize)]
struct VolumeStatus {
    pub uuid: Uuid,
    pub computer_name: String,
    pub domain_name: String,
    pub drive_letter: String,
    pub label: Option<String>,
    pub file_system: String,
    pub capacity: String,
    pub free_space: String,
    pub occupied_space: String,
    pub occupied_percentage: String,
}

#[get("/")]
fn index(database: &State<Database>) -> Template {
    let crit_volume = database
        .get_system_status_volume_crit()
        .unwrap_or_default()
        .len();
    Template::render("system_status", context! { crit_volume })
}

#[get("/volumes")]
fn volumes(database: &State<Database>) -> Template {
    let volumes: Vec<VolumeStatus> = database
        .get_system_status_volume_crit()
        .unwrap_or_default()
        .into_iter()
        .map(|(v, (c, os))| VolumeStatus {
            uuid: c.uuid,
            computer_name: os.computer_name,
            domain_name: os.domain.unwrap_or_default(),
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
    Template::render("system_status/volumes", context! { volumes })
}

pub fn routes() -> Vec<Route> {
    routes![index, volumes]
}
