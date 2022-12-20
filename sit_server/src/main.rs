#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;

use rocket::fs::FileServer;
use rocket::State;
use rocket_dyn_templates::{context, Template};

use crate::database::Database;

mod api_v1;
mod clients;
mod database;
mod profile;

#[get("/")]
fn index() -> Template {
    Template::render("index", context! {})
}

#[get("/software")]
fn software(database: &State<Database>) -> Template {
    let software_info = database.get_software_info().unwrap_or(vec![]);
    Template::render("software", context! { software: software_info })
}


#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .manage(Database::establish_connection())
        .attach(Template::fairing())
        .mount("/", routes![index, software])
        .mount("/clients/", routes![clients::index, clients::client, clients::profiles])
        .mount("/profile/", routes![profile::index, profile::profile])
        .mount("/api/v1/", routes![api_v1::register, api_v1::os, api_v1::hardware, api_v1::software, api_v1::profiles])
        .mount("/static", FileServer::from("static"))
        .launch()
        .await?;

    Ok(())
}
