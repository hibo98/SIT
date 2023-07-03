use rocket::{response::Redirect, Route, State};
use rocket_dyn_templates::{context, Template};

use crate::{auth::User, database::Database};

#[get("/")]
fn index(user: User) -> Template {
    Template::render("software/index", context! { user })
}

#[get("/software")]
fn software_list(database: &State<Database>, user: User) -> Template {
    let software_info = database.get_software_list().unwrap_or(vec![]);
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

#[get("/license")]
fn license_list(database: &State<Database>, user: User) -> Template {
    let license_info = database.get_license_list().unwrap_or(vec![]);
    Template::render(
        "software/license_list",
        context! { license: license_info, user },
    )
}

#[get("/license/<name>")]
fn license_computer(database: &State<Database>, name: String,  user: User) -> Template {
    let license_info = database.get_license_with_computers(&name).unwrap_or(vec![]);
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
        license_list,
        license_computer,
        catch_all
    ]
}
