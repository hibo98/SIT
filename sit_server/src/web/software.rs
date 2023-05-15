use rocket::{Route, State};
use rocket_dyn_templates::{context, Template};

use crate::database::Database;

#[get("/")]
fn index(database: &State<Database>) -> Template {
    let software_info = database.get_software_list().unwrap_or(vec![]);
    Template::render("software", context! { software: software_info })
}

#[get("/<id>")]
fn software(database: &State<Database>, id: i32) -> Template {
    let software_info = database.get_software_info(id);
    let software_versions = database.get_software_versions(id);
    if let (Ok(software_info), Ok(software_versions)) = (software_info, software_versions) {
        Template::render(
            "software_info",
            context! { software_info, software_versions },
        )
    } else {
        Template::render("software_info", context! {})
    }
}

#[get("/<id>/computer")]
fn software_computer(database: &State<Database>, id: i32) -> Template {
    let software_info = database.get_software_info(id);
    let computer_list = database.get_software_computer_list(id);
    if let (Ok(software_info), Ok(computer_list)) = (software_info, computer_list) {
        Template::render(
            "software_computer_list",
            context! { software_info, computer_list },
        )
    } else {
        Template::render("software_computer_list", context! {})
    }
}

#[get("/version/<id>", rank = 0)]
fn version(database: &State<Database>, id: i32) -> Template {
    let software_version = database.get_software_version(id);
    if let Ok(software_version) = software_version {
        let software_info = database.get_software_info(software_version.software_id);
        let software_versions_list = database.get_software_version_clients(id);
        if let (Ok(software_versions_list), Ok(software_info)) =
            (software_versions_list, software_info)
        {
            return Template::render(
                "software_version",
                context! { software_info, software_version, software_versions_list },
            );
        }
    }
    Template::render("software_version", context! {})
}

pub fn routes() -> Vec<Route> {
    routes![index, software, software_computer, version,]
}
