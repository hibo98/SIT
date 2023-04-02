use bigdecimal::ToPrimitive;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::{database::Database, display_util};

#[derive(Clone, Debug, Serialize)]
pub struct ProcessorCount {
    pub name: String,
    pub url_name: String,
    pub manufacturer: String,
    pub cores: Option<i64>,
    pub logical_cores: Option<i64>,
    pub clock_speed: Option<i64>,
    pub address_width: Option<i32>,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct MemoryCount {
    pub capacity: String,
    pub capacity_raw: u64,
    pub sticks: Option<i64>,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphicsCardCount {
    pub name: String,
    pub url_name: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiskCount {
    pub model: String,
    pub size: String,
    pub count: i64,
}

#[get("/")]
pub fn index() -> Template {
    Template::render("hardware", context! {})
}

#[get("/processors")]
pub fn processors(database: &State<Database>) -> Template {
    let processors = database.get_processors_count();
    if let Ok(processors) = processors {
        let processors: Vec<ProcessorCount> = processors
            .into_iter()
            .map(|p| ProcessorCount {
                url_name: urlencoding::encode(&p.name).into_owned(),
                name: p.name,
                manufacturer: p.manufacturer,
                cores: p.cores,
                logical_cores: p.logical_cores,
                clock_speed: p.clock_speed,
                address_width: p.address_width,
                count: p.count,
            })
            .collect();
        Template::render("hardware/processors", context! { processors })
    } else {
        Template::render("hardware/processors", context! {})
    }
}

#[get("/processors/<processor>")]
pub fn processor_clients(database: &State<Database>, processor: String) -> Template {
    let clients = database.get_processor_clients(&processor).unwrap_or(vec![]);
    Template::render(
        "hardware/clients",
        context! { clients, headline: processor },
    )
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
                capacity_raw: m
                    .capacity
                    .as_ref()
                    .map(|size| size.to_u64().unwrap_or_default())
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

#[get("/memory/<size>/<count>")]
pub fn memory_clients(database: &State<Database>, size: u64, count: i64) -> Template {
    let clients = database.get_memory_clients(size, count).unwrap_or(vec![]);
    Template::render(
        "hardware/clients",
        context! { clients, headline: format!("{}, {} Stick(s)", display_util::format_filesize_byte_iec(size as f64, 0), count) },
    )
}

#[get("/graphics_cards")]
pub fn graphics_cards(database: &State<Database>) -> Template {
    let graphics_cards = database.get_graphics_cards_count();
    if let Ok(graphics_cards) = graphics_cards {
        let graphics_cards: Vec<GraphicsCardCount> = graphics_cards
            .into_iter()
            .map(|gc| GraphicsCardCount {
                url_name: urlencoding::encode(&gc.name).into_owned(),
                name: gc.name,
                count: gc.count,
            })
            .collect();
        Template::render("hardware/graphics_cards", context! { graphics_cards })
    } else {
        Template::render("hardware/graphics_cards", context! {})
    }
}

#[get("/graphics_cards/<card>")]
pub fn graphics_cards_clients(database: &State<Database>, card: String) -> Template {
    let clients = database.get_graphics_card_clients(&card).unwrap_or(vec![]);
    Template::render(
        "hardware/clients",
        context! { clients, headline: card },
    )
}

#[get("/disks")]
pub fn disks(database: &State<Database>) -> Template {
    let disks = database.get_disks_count();
    if let Ok(disks) = disks {
        let disks: Vec<DiskCount> = disks
            .into_iter()
            .map(|d| DiskCount {
                model: d.model,
                size: d
                    .size
                    .as_ref()
                    .map(|size| {
                        size.to_f64()
                            .map(|size| display_util::format_filesize_byte(size, 0))
                            .unwrap_or_default()
                    })
                    .unwrap_or_default(),
                count: d.count,
            })
            .collect();
        Template::render("hardware/disks", context! { disks })
    } else {
        Template::render("hardware/disks", context! {})
    }
}

#[get("/models")]
pub fn models(database: &State<Database>) -> Template {
    let computer_models = database.get_computer_models_count().unwrap_or(vec![]);
    Template::render("hardware/models", context! { computer_models })
}

#[get("/network_adapters")]
pub fn network_adapters(database: &State<Database>) -> Template {
    let network_adapters = database.get_network_adapters_count().unwrap_or(vec![]);
    Template::render("hardware/network_adapters", context! { network_adapters })
}
