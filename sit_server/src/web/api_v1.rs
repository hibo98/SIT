use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use sit_lib::hardware::{BatteryStatus, HardwareInfo};
use sit_lib::licenses::LicenseBundle;
use sit_lib::os::UserProfiles;
use sit_lib::os::WinOsInfo;
use sit_lib::server::Register;
use sit_lib::software::SoftwareLibrary;
use sit_lib::system_status::VolumeList;
use sit_lib::task::Task;
use sit_lib::task::TaskBundle;
use sit_lib::task::TaskUpdate;
use uuid::Uuid;

use crate::database::Database;

#[post("/register", data = "<input>")]
async fn register(
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
async fn os(database: &State<Database>, uuid: Uuid, input: Json<WinOsInfo>) -> status::Custom<()> {
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
async fn hardware(
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
async fn software(
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
async fn profiles(
    database: &State<Database>,
    uuid: Uuid,
    input: Json<UserProfiles>,
) -> status::Custom<()> {
    match database.get_client(&uuid) {
        Ok(client) => match database.user_manager().update_profiles(client.id, input.0) {
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
async fn status_volumes(
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

#[post("/status/<uuid>/battery", data = "<input>")]
async fn status_battery(
    database: &State<Database>,
    uuid: Uuid,
    input: Json<BatteryStatus>,
) -> status::Custom<()> {
    match database.get_client(&uuid) {
        Ok(client) => match database.update_battery_status(client.id, input.0) {
            Ok(_) => status::Custom(Status::Ok, ()),
            Err(error) => {
                println!(
                    "[ERROR] In api_v1 /status/{}/battery update_battery_status {:?}",
                    uuid, error
                );
                status::Custom(Status::InternalServerError, ())
            }
        },
        Err(error) => {
            println!(
                "[ERROR] In api_v1 /status/{}/battery get_client {:?}",
                uuid, error
            );
            status::Custom(Status::InternalServerError, ())
        }
    }
}

#[post("/licenses/<uuid>", data = "<input>")]
async fn licenses(
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

#[get("/tasks/<uuid>")]
async fn tasks_get(database: &State<Database>, uuid: Uuid) -> status::Custom<Json<TaskBundle>> {
    match database.get_client(&uuid) {
        Ok(client) => match database.task_manager().get_new_tasks_for_client(client.id) {
            Ok(task_list) => status::Custom(
                Status::Ok,
                Json(TaskBundle {
                    tasks: task_list
                        .into_iter()
                        .map(|t| Task {
                            id: t.id,
                            task: t.task,
                            time_start: t.time_start.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc.offset_from_utc_datetime(&dt))),
                        })
                        .collect(),
                }),
            ),
            Err(error) => {
                println!(
                    "[ERROR] In api_v1 /licenses/{} update_license_keys {:?}",
                    uuid, error
                );
                status::Custom(
                    Status::InternalServerError,
                    Json(TaskBundle { tasks: vec![] }),
                )
            }
        },
        Err(error) => {
            println!("[ERROR] In api_v1 /tasks/{} get_client {:?}", uuid, error);
            status::Custom(
                Status::InternalServerError,
                Json(TaskBundle { tasks: vec![] }),
            )
        }
    }
}

#[post("/tasks/<uuid>", data = "<input>")]
async fn task_update(
    database: &State<Database>,
    uuid: Uuid,
    input: Json<TaskUpdate>,
) -> status::Custom<()> {
    match database.get_client(&uuid) {
        Ok(client) => match database.task_manager().update_task_status(client.id, input.0) {
            Ok(_) => status::Custom(Status::Ok, ()),
            Err(error) => {
                println!(
                    "[ERROR] In api_v1 /tasks/{} update_task_status {:?}",
                    uuid, error
                );
                status::Custom(Status::InternalServerError, ())
            }
        },
        Err(error) => {
            println!("[ERROR] In api_v1 /tasks/{} get_client {:?}", uuid, error);
            status::Custom(Status::InternalServerError, ())
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        register,
        os,
        hardware,
        software,
        profiles,
        status_volumes,
        status_battery,
        licenses,
        tasks_get,
        task_update,
    ]
}
