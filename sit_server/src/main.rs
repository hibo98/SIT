#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;

use rocket::fs::FileServer;
use rocket_dyn_templates::{context, Template};

use crate::database::Database;

mod api_v1;
mod clients;
mod database;
mod display_util;
mod ms_magic;
mod profile;
mod software;

#[get("/")]
fn index() -> Template {
    Template::render("index", context! {})
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .manage(Database::establish_connection())
        .attach(Template::fairing())
        .mount("/", routes![index])
        .mount(
            "/software/",
            routes![
                software::index,
                software::software,
                software::software_computer,
                software::version
            ],
        )
        .mount(
            "/clients/",
            routes![
                clients::index,
                clients::client,
                clients::profiles,
                clients::software
            ],
        )
        .mount("/profile/", routes![profile::index, profile::profile])
        .mount(
            "/api/v1/",
            routes![
                api_v1::register,
                api_v1::os,
                api_v1::hardware,
                api_v1::software,
                api_v1::profiles
            ],
        )
        .mount("/static", FileServer::from("static"))
        .launch()
        .await?;

    Ok(())
}
