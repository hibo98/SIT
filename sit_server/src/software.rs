use rocket::State;
use rocket_dyn_templates::{context, Template};

use crate::database::Database;

#[get("/")]
pub fn index(database: &State<Database>) -> Template {
    let software_info = database.get_software_list().unwrap_or(vec![]);
    Template::render("software", context! { software: software_info })
}

#[get("/<id>")]
pub fn software(database: &State<Database>, id: i32) -> Template {
    let software_info = database.get_software_info(id);
    if let Ok(software_info) = software_info {
        Template::render("software_info", context! { software: software_info })
    } else {
        Template::render("software_info", context! {})
    }
}

#[get("/<id>/computer")]
pub fn software_computer(database: &State<Database>, id: i32) -> Template {
    let result = database.get_software_computer_list(id);
    if let Ok((software_info, software_computer_list)) = result {
        Template::render(
            "software_computer_list",
            context! { software_info, software_computer_list },
        )
    } else {
        Template::render("software_computer_list", context! {})
    }
}

#[get("/version/<id>", rank = 0)]
pub fn version(database: &State<Database>, id: i32) -> Template {
    let result = database.get_software_version_list(id);
    if let Ok((software_info, software_version, software_versions_list)) = result {
        Template::render(
            "software_version",
            context! { software_info, software_version, software_versions_list },
        )
    } else {
        Template::render("software_version", context! {})
    }
}
