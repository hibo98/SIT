use rocket::State;
use rocket_dyn_templates::{context, Template};

use crate::database::Database;

#[get("/")]
pub fn index(database: &State<Database>) -> Template {
    let profiles = database.get_profiles().unwrap_or(vec![]);
    Template::render("profiles", context! { profiles })
}

#[get("/<sid>")]
pub fn profile(database: &State<Database>, sid: String) -> Template {
    let profile = database.get_profile_info(sid);
    if let Ok(profile) = profile {
        Template::render("profile", context! { profile })
    } else {
        Template::render("profile", context! {})
    }
}
