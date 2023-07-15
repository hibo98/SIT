#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;

use rocket::{fs::FileServer, response::Redirect};
use rocket_dyn_templates::{context, Template};

use crate::{auth::User, database::Database};

mod auth;
mod database;
mod web;

#[get("/")]
fn index(user: User) -> Template {
    Template::render("index", context! { user })
}

#[get("/", rank = 10)]
fn non_user_index() -> Redirect {
    Redirect::to(uri!("/auth/login"))
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .manage(Database::establish_connection())
        .attach(Template::fairing())
        .mount("/", routes![index, non_user_index])
        .mount("/api/v1/", web::api_v1::routes())
        .mount("/auth", web::auth::routes())
        .mount("/clients/", web::clients::routes())
        .mount("/hardware/", web::hardware::routes())
        .mount("/profile/", web::profile::routes())
        .mount("/settings/", web::settings::routes())
        .mount("/software/", web::software::routes())
        .mount("/static", FileServer::from("static"))
        .mount("/system-status", web::system_status::routes())
        .launch()
        .await?;

    Ok(())
}
