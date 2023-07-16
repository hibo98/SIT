use rocket::{form::Form, response::Redirect, Route, State};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::{auth::User, database::Database};

#[derive(Clone, Debug, Serialize)]
pub struct SoftwareInfo {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SoftwareVersion {
    pub id: i32,
    pub name: String,
    pub version: String,
}

#[derive(FromForm)]
struct Login<'r> {
    username: &'r str,
    password: &'r str,
}

#[get("/")]
fn index(user: User) -> Template {
    Template::render("settings/index", context! { user })
}

#[get("/users")]
fn users(db: &State<Database>, user: User) -> Template {
    let users_result = db.get_auth_users();
    if let Ok(auth_users) = users_result {
        Template::render("settings/users", context! { user, auth_users })
    } else {
        Template::render("settings/users", context! { user })
    }
}

#[get("/users/new")]
fn new_user(_user: User) -> Template {
    Template::render("settings/users_new", context! {})
}

#[post("/users/new", data = "<user>")]
fn post_new_user(db: &State<Database>, user: Form<Login<'_>>, _guard: User) -> Redirect {
    let result = crate::auth::create_new_user(db, user.username, user.password);
    if result.is_err() {
        Redirect::to(uri!("/settings", new_user))
        // TODO: Add error cause
    } else {
        Redirect::to(uri!("/settings", users))
    }
}

#[get("/service")]
fn service_index(user: User) -> Template {
    Template::render("settings/service", context! { user })
}

#[get("/service/software")]
fn service_software(db: &State<Database>, user: User) -> Template {
    let mut delete_software_version: Vec<SoftwareVersion> = vec![];
    let mut delete_software: Vec<SoftwareInfo> = vec![];
    let software_list = db.get_software_list().unwrap_or(vec![]);
    for software in software_list {
        let versions = db.get_software_versions(software.id).unwrap_or(vec![]);
        for version in &versions {
            if version.count == 0 {
                delete_software_version.push(SoftwareVersion {
                    id: version.id,
                    name: software.name.clone(),
                    version: version.version.clone(),
                });
            }
        }
        if versions.is_empty() {
            delete_software.push(SoftwareInfo {
                id: software.id,
                name: software.name,
            });
        }
    }
    delete_software_version.sort_by_key(|f| f.name.clone());
    delete_software.sort_by_key(|f| f.name.clone());
    Template::render(
        "settings/service_software",
        context! { delete_software_version, delete_software, user },
    )
}

#[get("/service/software/cleanup/version")]
fn service_software_cleanup_version(db: &State<Database>, _user: User) -> Redirect {
    let software_list = db.get_software_list().unwrap_or(vec![]);
    for software in software_list {
        let versions = db.get_software_versions(software.id).unwrap_or(vec![]);
        for version in &versions {
            if version.count == 0 {
                let _ = db.delete_software_version(version.id);
            }
        }
    }
    Redirect::to(uri!("/settings", service_software))
}

#[get("/service/software/cleanup/info")]
fn service_software_cleanup_list(db: &State<Database>, _user: User) -> Redirect {
    let software_list = db.get_software_list().unwrap_or(vec![]);
    for software in software_list {
        let versions = db.get_software_versions(software.id);
        if let Ok(versions) = versions {
            if versions.is_empty() {
                let _ = db.delete_software_info(software.id);
            }
        }
    }
    Redirect::to(uri!("/settings", service_software))
}

pub fn routes() -> Vec<Route> {
    routes![
        index,
        users,
        new_user,
        post_new_user,
        service_index,
        service_software,
        service_software_cleanup_version,
        service_software_cleanup_list,
    ]
}
