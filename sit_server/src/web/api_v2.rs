use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{Route, State};
use uuid::Uuid;
use sit_lib::hardware::HardwareInfoV2;
use crate::database::Database;

#[post("/hardware/<uuid>", data = "<input>")]
async fn hardware(
    database: &State<Database>,
    uuid: Uuid,
    input: Json<HardwareInfoV2>,
) -> status::Custom<()> {
    match database.get_client(&uuid) {
        Ok(client) => match database.create_hardware_info_v2(client.id, input.0) {
            Ok(_) => status::Custom(Status::Ok, ()),
            Err(error) => {
                println!(
                    "[ERROR] In api_v2 /hardware/{} create_hardware_info {:?}",
                    uuid, error
                );
                status::Custom(Status::InternalServerError, ())
            }
        },
        Err(error) => {
            println!(
                "[ERROR] In api_v2 /hardware/{} get_client {:?}",
                uuid, error
            );
            status::Custom(Status::InternalServerError, ())
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        hardware,
    ]
}
