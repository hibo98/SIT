use bigdecimal::ToPrimitive;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::{database::Database, display_util};

#[derive(Clone, Debug, Serialize)]
pub struct MemoryCount {
    pub capacity: String,
    pub sticks: Option<i64>,
    pub count: i64,
}

#[get("/")]
pub fn index() -> Template {
    Template::render("hardware", context! {})
}

#[get("/processors")]
pub fn processors(database: &State<Database>) -> Template {
    let processors = database.get_processors_count().unwrap_or(vec![]);
    Template::render("hardware/processors", context! { processors })
}

#[get("/memory")]
pub fn memory(database: &State<Database>) -> Template {
    let memorys = database.get_memorys_count();
    if let Ok(memorys) = memorys {
        let memorys: Vec<MemoryCount> = memorys
            .into_iter()
            .map(|m| MemoryCount {
                capacity: m
                    .capacity
                    .as_ref()
                    .map(|size| {
                        size.to_f64()
                            .map(|size| display_util::format_filesize_byte_iec(size, 0))
                            .unwrap_or_default()
                    })
                    .unwrap_or_default(),
                sticks: m.sticks,
                count: m.count,
            })
            .collect();
        Template::render("hardware/memory", context! { memorys })
    } else {
        Template::render("hardware/memory", context! {})
    }
}

#[get("/graphics_cards")]
pub fn graphics_cards(database: &State<Database>) -> Template {
    let graphics_cards = database.get_graphics_cards_count().unwrap_or(vec![]);
    Template::render("hardware/graphics_cards", context! { graphics_cards })
}

#[get("/disks")]
pub fn disks(database: &State<Database>) -> Template {
    let disks = database.get_disks().unwrap_or(vec![]);
    Template::render("hardware/disks", context! { disks })
}

#[get("/models")]
pub fn models(database: &State<Database>) -> Template {
    let computer_models = database.get_computer_models().unwrap_or(vec![]);
    Template::render("hardware/models", context! { computer_models })
}

#[get("/bios")]
pub fn bios(database: &State<Database>) -> Template {
    let bios_list = database.get_bios_list().unwrap_or(vec![]);
    Template::render("hardware/bios", context! { bios_list })
}

#[get("/network_adapters")]
pub fn network_adapters(database: &State<Database>) -> Template {
    let network_adapters = database.get_network_adapters().unwrap_or(vec![]);
    Template::render("hardware/network_adapters", context! { network_adapters })
}
