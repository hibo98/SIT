use bigdecimal::ToPrimitive;
use rocket::{response::Redirect, Route, State};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::{auth::User, database::Database};

use super::display_util;

#[derive(Clone, Debug, Serialize)]
struct ProcessorCount {
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
struct MemoryCount {
    pub capacity: String,
    pub capacity_raw: u64,
    pub sticks: Option<i64>,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize)]
struct GraphicsCardCount {
    pub name: String,
    pub url_name: String,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize)]
struct DiskCount {
    pub model: String,
    pub url_model: String,
    pub size: String,
    pub size_raw: u64,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize)]
struct ComputerModelCount {
    pub manufacturer: String,
    pub url_manufacturer: String,
    pub model_family: String,
    pub url_model_family: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
struct NetworkAdapterCount {
    pub name: String,
    pub url_name: String,
    pub count: i64,
}

#[get("/")]
fn index(user: User) -> Template {
    Template::render("hardware/index", context! { user })
}

#[get("/processors")]
fn processors(database: &State<Database>, user: User) -> Template {
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
        Template::render("hardware/processors", context! { processors, user })
    } else {
        Template::render("hardware/processors", context! {})
    }
}

#[get("/processors/<processor>")]
fn processor_clients(database: &State<Database>, processor: String, user: User) -> Template {
    let clients = database.get_processor_clients(&processor).unwrap_or_default();
    Template::render(
        "hardware/clients",
        context! { clients, headline: processor, user },
    )
}

#[get("/memory")]
fn memory(database: &State<Database>, user: User) -> Template {
    let memorys = database.get_memorys_count();
    if let Ok(memorys) = memorys {
        let memorys: Vec<MemoryCount> = memorys
            .into_iter()
            .map(|m| MemoryCount {
                capacity: display_util::format_option_big_decimal(
                    &m.capacity,
                    display_util::format_filesize_byte_iec,
                ),
                capacity_raw: m
                    .capacity
                    .as_ref()
                    .map(|size| size.to_u64().unwrap_or_default())
                    .unwrap_or_default(),
                sticks: m.sticks,
                count: m.count,
            })
            .collect();
        Template::render("hardware/memory", context! { memorys, user })
    } else {
        Template::render("hardware/memory", context! {})
    }
}

#[get("/memory/<size>/<count>")]
fn memory_clients(database: &State<Database>, size: u64, count: i64, user: User) -> Template {
    let clients = database.get_memory_clients(size, count).unwrap_or_default();
    Template::render(
        "hardware/clients",
        context! { clients, headline: format!("{}, {} Stick(s)", display_util::format_filesize_byte_iec(size as f64, 0), count), user },
    )
}

#[get("/graphics_cards")]
fn graphics_cards(database: &State<Database>, user: User) -> Template {
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
        Template::render("hardware/graphics_cards", context! { graphics_cards, user })
    } else {
        Template::render("hardware/graphics_cards", context! {})
    }
}

#[get("/graphics_cards/<card>")]
fn graphics_card_clients(database: &State<Database>, card: String, user: User) -> Template {
    let clients = database.get_graphics_card_clients(&card).unwrap_or_default();
    Template::render(
        "hardware/clients",
        context! { clients, headline: card, user },
    )
}

#[get("/disks")]
fn disks(database: &State<Database>, user: User) -> Template {
    let disks = database.get_disks_count();
    if let Ok(disks) = disks {
        let disks: Vec<DiskCount> = disks
            .into_iter()
            .map(|d| DiskCount {
                url_model: urlencoding::encode(&d.model).into_owned(),
                model: d.model,
                size: display_util::format_option_big_decimal(
                    &d.size,
                    display_util::format_filesize_byte,
                ),
                size_raw: d
                    .size
                    .as_ref()
                    .map(|size| size.to_u64().unwrap_or_default())
                    .unwrap_or_default(),
                count: d.count,
            })
            .collect();
        Template::render("hardware/disks", context! { disks, user })
    } else {
        Template::render("hardware/disks", context! {})
    }
}

#[get("/disks/<model>/<size>")]
fn disk_clients(database: &State<Database>, model: String, size: u64, user: User) -> Template {
    let clients = database.get_disk_clients(&model, size).unwrap_or_default();
    Template::render(
        "hardware/clients",
        context! { clients, headline: format!("{}, {}", model, display_util::format_filesize_byte(size as f64, 0)), user },
    )
}

#[get("/models")]
fn models(database: &State<Database>, user: User) -> Template {
    let computer_models = database.get_computer_models_count();
    if let Ok(computer_models) = computer_models {
        let computer_models: Vec<ComputerModelCount> = computer_models
            .into_iter()
            .map(|m| ComputerModelCount {
                url_manufacturer: urlencoding::encode(&m.manufacturer).into_owned(),
                url_model_family: urlencoding::encode(&m.model_family).into_owned(),
                manufacturer: m.manufacturer,
                model_family: m.model_family,
                count: m.count,
            })
            .collect();
        Template::render("hardware/models", context! { computer_models, user })
    } else {
        Template::render("hardware/models", context! {})
    }
}

#[get("/models/<manufacturer>/<model>")]
fn model_clients(
    database: &State<Database>,
    manufacturer: String,
    model: String,
    user: User,
) -> Template {
    let clients = database
        .get_computer_model_clients(&model, &manufacturer)
        .unwrap_or_default();
    Template::render(
        "hardware/clients",
        context! { clients, headline: format!("{}, {}", manufacturer, model), user },
    )
}

#[get("/network_adapters")]
fn network_adapters(database: &State<Database>, user: User) -> Template {
    let network_adapters = database.get_network_adapters_count();
    if let Ok(network_adapters) = network_adapters {
        let network_adapters: Vec<NetworkAdapterCount> = network_adapters
            .into_iter()
            .map(|na| NetworkAdapterCount {
                url_name: urlencoding::encode(&na.name).into_owned(),
                name: na.name,
                count: na.count,
            })
            .collect();
        Template::render(
            "hardware/network_adapters",
            context! { network_adapters, user },
        )
    } else {
        Template::render("hardware/network_adapters", context! {})
    }
}

#[get("/network_adapters/<name>")]
fn network_adapter_clients(database: &State<Database>, name: String, user: User) -> Template {
    let clients = database
        .get_network_adapter_clients(&name)
        .unwrap_or_default();
    Template::render(
        "hardware/clients",
        context! { clients, headline: name, user },
    )
}

#[get("/<_..>", rank = 10)]
fn catch_all() -> Redirect {
    Redirect::to(uri!("/auth/login"))
}

pub fn routes() -> Vec<Route> {
    routes![
        index,
        processors,
        processor_clients,
        memory,
        memory_clients,
        graphics_cards,
        graphics_card_clients,
        disks,
        disk_clients,
        models,
        model_clients,
        network_adapters,
        network_adapter_clients,
        catch_all,
    ]
}
