use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use sit_lib::hardware::HardwareInfo;
use sit_lib::licenses::LicenseBundle;
use sit_lib::os::UserProfiles;
use sit_lib::os::WinOsInfo;
use sit_lib::server::Register;
use sit_lib::software::SoftwareLibrary;
use sit_lib::system_status::VolumeList;
use uuid::Uuid;

use crate::database::Database;

#[post("/register", data = "<input>")]
pub async fn register(
    database: &State<Database>,
    input: Json<Register>,
) -> status::Custom<Json<Register>> {
    let uuid = input.uuid.unwrap_or_else(Uuid::new_v4);
    let computer_name = input.name.clone();
    match database.create_client(&uuid) {
        Ok(client) => match database.create_os_info(&client, &computer_name) {
            Ok(_) => status::Custom(
                Status::Created,
                Json(Register {
                    name: computer_name,
                    uuid: Some(uuid),
                }),
            ),
            Err(error) => {
                println!("[ERROR] In api_v1 /register create_os_info {:?}", error);
                status::Custom(
                    Status::InternalServerError,
                    Json(Register {
                        name: computer_name,
                        uuid: None,
                    }),
                )
            }
        },
        Err(error) => {
            println!("[ERROR] In api_v1 /register create_client {:?}", error);
            status::Custom(
                Status::InternalServerError,
                Json(Register {
                    name: computer_name,
                    uuid: None,
                }),
            )
        }
    }
}

#[post("/os/<uuid>", data = "<input>")]
pub async fn os(
    database: &State<Database>,
    uuid: Uuid,
    input: Json<WinOsInfo>,
) -> status::Custom<()> {
    match database.get_client(&uuid) {
        Ok(client) => match database.update_os_info(client.id, input.0) {
            Ok(_) => status::Custom(Status::Ok, ()),
            Err(error) => {
                println!("[ERROR] In api_v1 /os/{} update_os_info {:?}", uuid, error);
                status::Custom(Status::InternalServerError, ())
            }
        },
        Err(error) => {
            println!("[ERROR] In api_v1 /os/{} get_client {:?}", uuid, error);
            status::Custom(Status::InternalServerError, ())
        }
    }
}

#[post("/hardware/<uuid>", data = "<input>")]
pub async fn hardware(
    database: &State<Database>,
    uuid: Uuid,
    input: Json<HardwareInfo>,
) -> status::Custom<()> {
    match database.get_client(&uuid) {
        Ok(client) => match database.create_hardware_info(client.id, input.0) {
            Ok(_) => status::Custom(Status::Ok, ()),
            Err(error) => {
                println!(
                    "[ERROR] In api_v1 /hardware/{} create_hardware_info {:?}",
                    uuid, error
                );
                status::Custom(Status::InternalServerError, ())
            }
        },
        Err(error) => {
            println!(
                "[ERROR] In api_v1 /hardware/{} get_client {:?}",
                uuid, error
            );
            status::Custom(Status::InternalServerError, ())
        }
    }
}

#[post("/software/<uuid>", data = "<input>")]
pub async fn software(
    database: &State<Database>,
    uuid: Uuid,
    input: Json<SoftwareLibrary>,
) -> status::Custom<()> {
    match database.get_client(&uuid) {
        Ok(client) => match database.update_software_lib(client.id, input.0) {
            Ok(_) => status::Custom(Status::Ok, ()),
            Err(error) => {
                println!(
                    "[ERROR] In api_v1 /software/{} update_software_lib {:?}",
                    uuid, error
                );
                status::Custom(Status::InternalServerError, ())
            }
        },
        Err(error) => {
            println!(
                "[ERROR] In api_v1 /software/{} get_client {:?}",
                uuid, error
            );
            status::Custom(Status::InternalServerError, ())
        }
    }
}

#[post("/profiles/<uuid>", data = "<input>")]
pub async fn profiles(
    database: &State<Database>,
    uuid: Uuid,
    input: Json<UserProfiles>,
) -> status::Custom<()> {
    match database.get_client(&uuid) {
        Ok(client) => match database.update_profiles(client.id, input.0) {
            Ok(_) => status::Custom(Status::Ok, ()),
            Err(error) => {
                println!(
                    "[ERROR] In api_v1 /profiles/{} update_profiles {:?}",
                    uuid, error
                );
                status::Custom(Status::InternalServerError, ())
            }
        },
        Err(error) => {
            println!(
                "[ERROR] In api_v1 /profiles/{} get_client {:?}",
                uuid, error
            );
            status::Custom(Status::InternalServerError, ())
        }
    }
}

#[post("/status/<uuid>/volumes", data = "<input>")]
pub async fn status_volumes(
    database: &State<Database>,
    uuid: Uuid,
    input: Json<VolumeList>,
) -> status::Custom<()> {
    match database.get_client(&uuid) {
        Ok(client) => match database.update_status_volumes(client.id, input.0) {
            Ok(_) => status::Custom(Status::Ok, ()),
            Err(error) => {
                println!(
                    "[ERROR] In api_v1 /status/{}/volumes update_status_volumes {:?}",
                    uuid, error
                );
                status::Custom(Status::InternalServerError, ())
            }
        },
        Err(error) => {
            println!(
                "[ERROR] In api_v1 /status/{}/volumes get_client {:?}",
                uuid, error
            );
            status::Custom(Status::InternalServerError, ())
        }
    }
}

#[post("/licenses/<uuid>", data = "<input>")]
pub async fn licenses(
    database: &State<Database>,
    uuid: Uuid,
    input: Json<LicenseBundle>,
) -> status::Custom<()> {
    match database.get_client(&uuid) {
        Ok(client) => match database.update_license_keys(client.id, input.0) {
            Ok(_) => status::Custom(Status::Ok, ()),
            Err(error) => {
                println!(
                    "[ERROR] In api_v1 /licenses/{} update_license_keys {:?}",
                    uuid, error
                );
                status::Custom(Status::InternalServerError, ())
            }
        },
        Err(error) => {
            println!(
                "[ERROR] In api_v1 /licenses/{} get_client {:?}",
                uuid, error
            );
            status::Custom(Status::InternalServerError, ())
        }
    }
}
