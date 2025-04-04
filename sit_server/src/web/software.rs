use rocket::{response::Redirect, Route, State};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::{auth::User, database::Database};

#[derive(Clone, Debug, Serialize)]
pub struct SoftwareVersionWithCount {
    pub id: i32,
    pub software_id: i32,
    pub version: String,
    pub count: i64,
}

#[get("/")]
fn index(user: User) -> Template {
    Template::render("software/index", context! { user })
}

#[get("/software")]
fn software_list(database: &State<Database>, user: User) -> Template {
    let software_info = database.get_software_list().unwrap_or_default();
    Template::render(
        "software/software_list",
        context! { software: software_info, user },
    )
}

#[get("/software/<id>")]
fn software(database: &State<Database>, id: i32, user: User) -> Template {
    let software_info = database.get_software_info(id);
    let software_versions = database.get_software_versions(id);
    if let (Ok(software_info), Ok(software_versions)) = (software_info, software_versions) {
        let software_versions: Vec<SoftwareVersionWithCount> = software_versions
            .into_iter()
            .map(|sv| SoftwareVersionWithCount {
                id: sv.id,
                software_id: sv.software_id,
                version: if sv.version.is_empty() { "<_version>".to_string() } else { sv.version },
                count: sv.count,
            })
            .collect();
        Template::render(
            "software/software",
            context! { software_info, software_versions, user },
        )
    } else {
        Template::render("software/software", context! {})
    }
}

#[get("/software/<id>/computer")]
fn software_computer(database: &State<Database>, id: i32, user: User) -> Template {
    let software_info = database.get_software_info(id);
    let computer_list = database.get_software_computer_list(id);
    if let (Ok(software_info), Ok(computer_list)) = (software_info, computer_list) {
        Template::render(
            "software/software_computer_list",
            context! { software_info, computer_list, user },
        )
    } else {
        Template::render("software/software_computer_list", context! {})
    }
}

#[get("/software/<_>/version/<id>")]
fn software_version(database: &State<Database>, id: i32, user: User) -> Template {
    let software_version = database.get_software_version(id);
    if let Ok(software_version) = software_version {
        let software_info = database.get_software_info(software_version.software_id);
        let software_versions_list = database.get_software_version_clients(id);
        if let (Ok(software_versions_list), Ok(software_info)) =
            (software_versions_list, software_info)
        {
            return Template::render(
                "software/software_version",
                context! { software_info, software_version, software_versions_list, user },
            );
        }
    }
    Template::render("software/software_version", context! {})
}

#[get("/os")]
fn os_list(database: &State<Database>, user: User) -> Template {
    let os_list = database.get_os_list().unwrap_or_default();
    Template::render(
        "software/os_list",
        context! { os: os_list, user },
    )
}

#[get("/os/<name>")]
fn os_versions(database: &State<Database>, name: String, user: User) -> Template {
    let os_version = database.get_os_versions(name);
    if let Ok(os_version) = os_version {
        Template::render(
            "software/os_version",
            context! { os_version, user },
        )
    } else {
        Template::render("software/os_version", context! {})
    }
}

#[get("/os/<name>/computer")]
fn os_computer(database: &State<Database>, name: String, user: User) -> Template {
    let os_computer = database.get_os_client_list(name.clone());
    if let Ok(os_computer) = os_computer {
        Template::render(
            "software/os_computer",
            context! { os_name: name, os_computer: os_computer, user },
        )
    } else {
        Template::render("software/os_computer", context! {})
    }
}

#[get("/os/<name>/<version>")]
fn os_version_computer(database: &State<Database>, name: String, version: String, user: User) -> Template {
    let os_version_computer = database.get_os_version_client_list(name, version);
    if let Ok(os_version_computer) = os_version_computer {
        Template::render(
            "software/os_version_computer",
            context! { os_name: os_version_computer.os, os_verison: os_version_computer.os_version, os_version_computer: os_version_computer.list, user },
        )
    } else {
        Template::render("software/os_version_computer", context! {})
    }
}

#[get("/license")]
fn license_list(database: &State<Database>, user: User) -> Template {
    let license_info = database.get_license_list().unwrap_or_default();
    Template::render(
        "software/license_list",
        context! { license: license_info, user },
    )
}

#[get("/license/<name>")]
fn license_computer(database: &State<Database>, name: String, user: User) -> Template {
    let license_info = database.get_license_with_computers(&name).unwrap_or_default();
    Template::render(
        "software/license_computer",
        context! { license: license_info, user },
    )
}

#[get("/<_..>", rank = 10)]
fn catch_all() -> Redirect {
    Redirect::to(uri!("/auth/login"))
}

pub fn routes() -> Vec<Route> {
    routes![
        index,
        software_list,
        software,
        software_computer,
        software_version,
        os_list,
        os_versions,
        os_computer,
        os_version_computer,
        license_list,
        license_computer,
        catch_all
    ]
}
