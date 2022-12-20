use rocket::State;
use rocket_dyn_templates::{context, Template};
use uuid::Uuid;

use crate::database::Database;

#[get("/")]
pub fn index(database: &State<Database>) -> Template {
    let client_info = database.get_clients_info().unwrap_or(vec![]);
    Template::render("clients", context! { clients: client_info })
}

#[get("/<uuid>")]
pub fn client(database: &State<Database>, uuid: Uuid) -> Template {
    let client_info = database.get_client_info(uuid);
    if let Ok(client_info) = client_info {
        Template::render("client", context! { client: client_info })
    } else {
        Template::render("client", context! {})
    }
}

#[get("/<uuid>/profiles")]
pub fn profiles(database: &State<Database>, uuid: Uuid) -> Template {
    let client_profiles = database.get_client_profiles(uuid);
    if let Ok(client_profiles) = client_profiles {
        Template::render("client_profiles", context! { profiles: client_profiles })
    } else {
        Template::render("client_profiles", context! {})
    }
}
