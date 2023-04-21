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
mod hardware;
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
            "/hardware/",
            routes![
                hardware::index,
                hardware::processors,
                hardware::processor_clients,
                hardware::memory,
                hardware::memory_clients,
                hardware::graphics_cards,
                hardware::graphics_card_clients,
                hardware::disks,
                hardware::disk_clients,
                hardware::models,
                hardware::model_clients,
                hardware::network_adapters,
                hardware::network_adapter_clients,
            ],
        )
        .mount(
            "/software/",
            routes![
                software::index,
                software::software,
                software::software_computer,
                software::version,
            ],
        )
        .mount(
            "/clients/",
            routes![
                clients::index,
                clients::client,
                clients::profiles,
                clients::software,
                clients::hardware,
                clients::status,
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
                api_v1::profiles,
                api_v1::status_volumes,
            ],
        )
        .mount("/static", FileServer::from("static"))
        .launch()
        .await?;

    Ok(())
}
