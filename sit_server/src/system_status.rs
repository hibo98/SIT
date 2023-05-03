use rocket::State;
use rocket_dyn_templates::{context, Template};

use crate::database::Database;

#[get("/")]
pub fn index(database: &State<Database>) -> Template {
    //TOOD: Retrive info about how many Problems in the categories
    Template::render("system_status", context! {})
}

#[get("/volumes")]
pub fn volumes(database: &State<Database>) -> Template {
    let volumes = database.get_system_status_volume_crit().unwrap_or_default();
    Template::render("system_status/volumes", context! { volumes })
}
