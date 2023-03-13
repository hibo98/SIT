use rocket::State;
use rocket_dyn_templates::{context, Template};

use crate::database::Database;


#[get("/")]
pub fn index(database: &State<Database>) -> Template {
    let processors = database.get_processors().unwrap_or(vec![]);
    let memorys = database.get_memorys().unwrap_or(vec![]);
    let graphics_cards = database.get_graphics_cards().unwrap_or(vec![]);
    let disks = database.get_disks().unwrap_or(vec![]);
    let computer_models = database.get_computer_models().unwrap_or(vec![]);
    let bios_list = database.get_bios_list().unwrap_or(vec![]);
    let network_adapters = database.get_network_adapters().unwrap_or(vec![]);
    Template::render("hardware", context! { processors, memorys, graphics_cards, disks, computer_models, bios_list, network_adapters })
}