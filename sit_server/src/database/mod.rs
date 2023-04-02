extern crate dotenv;

use std::env;

use anyhow::Result;
use bigdecimal::BigDecimal;
use diesel::dsl::{count, count_star, max, sum};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_types::{BigInt, Nullable};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use sit_lib::hardware::HardwareInfo;
use sit_lib::os::{UserProfiles, WinOsInfo};
use sit_lib::software::SoftwareLibrary;
use uuid::Uuid;

use crate::database::model::*;
use crate::database::schema::*;

mod model;
mod schema;

sql_function! { fn coalesce(x: Nullable<BigInt>, y: BigInt) -> BigInt; }

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub struct Database {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl Database {
    pub fn establish_connection() -> Database {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        PgConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connection to {database_url}"))
            .run_pending_migrations(MIGRATIONS)
            .expect("Migrations failed");

        let manager = ConnectionManager::<PgConnection>::new(database_url);

        let pool = Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .expect("Could not build connection pool");

        Database { pool }
    }

    pub fn create_client(&self, uuid: &Uuid) -> Result<Client> {
        let mut conn = self.pool.get()?;
        Ok(diesel::insert_into(client::table)
            .values(NewClient { uuid })
            .on_conflict(client::uuid)
            .do_update()
            .set(client::uuid.eq(uuid))
            .get_result(&mut conn)?)
    }

    pub fn get_client(&self, uuid: &Uuid) -> Result<Client> {
        let mut conn = self.pool.get()?;
        Ok(client::table
            .filter(client::uuid.eq(uuid))
            .get_result(&mut conn)?)
    }

    pub fn create_os_info(&self, client: &Client, computer_name: &str) -> Result<OsInfo> {
        let mut conn = self.pool.get()?;
        Ok(diesel::insert_into(os_info::table)
            .values(NewOsInfo {
                client_id: &client.id,
                computer_name,
            })
            .on_conflict(os_info::client_id)
            .do_update()
            .set(os_info::computer_name.eq(computer_name))
            .get_result(&mut conn)?)
    }

    pub fn update_os_info(&self, client_id: i32, win_os_info: WinOsInfo) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(diesel::update(os_info::table)
            .set(UpdateOsInfo {
                os: Some(&win_os_info.operating_system),
                os_version: Some(&win_os_info.os_version),
                computer_name: Some(&win_os_info.computer_name),
                domain: Some(&win_os_info.domain),
            })
            .filter(os_info::client_id.eq(client_id))
            .execute(&mut conn)?)
    }

    pub fn create_hardware_info(&self, client_id: i32, hardware_info: HardwareInfo) -> Result<()> {
        let mut conn = self.pool.get()?;
        diesel::insert_into(computer_model::table)
            .values(NewComputerModel {
                client_id: &client_id,
                manufacturer: &hardware_info.model.manufacturer,
                model_family: &hardware_info.model.model_family,
                serial_number: &hardware_info.model.serial_number,
            })
            .on_conflict(computer_model::client_id)
            .do_update()
            .set((
                computer_model::manufacturer.eq(&hardware_info.model.manufacturer),
                computer_model::model_family.eq(&hardware_info.model.model_family),
                computer_model::serial_number.eq(&hardware_info.model.serial_number),
            ))
            .execute(&mut conn)?;
        diesel::delete(memory_stick::table.filter(memory_stick::client_id.eq(client_id)))
            .execute(&mut conn)?;
        for stick in hardware_info.memory.sticks {
            diesel::insert_into(memory_stick::table)
                .values(NewMemoryStick {
                    client_id: &client_id,
                    capacity: &BigDecimal::from(stick.capacity),
                    bank_label: &stick.bank_label,
                })
                .execute(&mut conn)?;
        }
        diesel::insert_into(processor::table)
            .values(NewProcessor {
                client_id: &client_id,
                name: &hardware_info.processor.name,
                manufacturer: &hardware_info.processor.manufacturer,
                cores: &(hardware_info.processor.cores as i64),
                logical_cores: &(hardware_info.processor.logical_cores as i64),
                clock_speed: &(hardware_info.processor.clock_speed as i64),
                address_width: &(hardware_info.processor.address_width as i32),
            })
            .on_conflict(processor::client_id)
            .do_update()
            .set((
                processor::name.eq(&hardware_info.processor.name),
                processor::manufacturer.eq(&hardware_info.processor.manufacturer),
                processor::cores.eq(&(hardware_info.processor.cores as i64)),
                processor::logical_cores.eq(&(hardware_info.processor.logical_cores as i64)),
                processor::clock_speed.eq(&(hardware_info.processor.clock_speed as i64)),
                processor::address_width.eq(&(hardware_info.processor.address_width as i32)),
            ))
            .execute(&mut conn)?;
        diesel::delete(disks::table.filter(disks::client_id.eq(client_id))).execute(&mut conn)?;
        for disk in hardware_info.disks.drives {
            diesel::insert_into(disks::table)
                .values(NewDisk {
                    client_id: &client_id,
                    model: &disk.model,
                    serial_number: &disk.serial_number,
                    size: Some(BigDecimal::from(disk.size)),
                    device_id: &disk.device_id,
                    status: &disk.status,
                    media_type: &disk.media_type,
                })
                .execute(&mut conn)?;
        }
        diesel::delete(network_adapter::table.filter(network_adapter::client_id.eq(client_id)))
            .execute(&mut conn)?;
        for na in hardware_info.network.adapter {
            let db_na: NetworkAdapter = diesel::insert_into(network_adapter::table)
                .values(NewNetworkAdapter {
                    client_id: &client_id,
                    name: &na.name,
                    mac_address: na.mac_address.as_ref(),
                })
                .get_result(&mut conn)?;
            diesel::delete(
                network_adapter_ip::table.filter(network_adapter_ip::adapter_id.eq(db_na.id)),
            )
            .execute(&mut conn)?;
            if let Some(ips) = na.ip_addresses {
                for nai in ips {
                    diesel::insert_into(network_adapter_ip::table)
                        .values(NewNetworkAdapterIp {
                            adapter_id: &db_na.id,
                            ip: &nai,
                        })
                        .execute(&mut conn)?;
                }
            }
        }
        diesel::insert_into(graphics_card::table)
            .values(NewGraphicsCard {
                client_id: &client_id,
                name: &hardware_info.graphics.name,
            })
            .on_conflict(graphics_card::client_id)
            .do_update()
            .set(graphics_card::name.eq(&hardware_info.graphics.name))
            .execute(&mut conn)?;
        diesel::insert_into(bios::table)
            .values(NewBios {
                client_id: &client_id,
                name: &hardware_info.bios.name,
                manufacturer: &hardware_info.bios.manufacturer,
                version: &hardware_info.bios.version,
            })
            .on_conflict(bios::client_id)
            .do_update()
            .set((
                bios::name.eq(&hardware_info.bios.name),
                bios::manufacturer.eq(&hardware_info.bios.manufacturer),
                bios::version.eq(&hardware_info.bios.version),
            ))
            .execute(&mut conn)?;
        Ok(())
    }

    pub fn update_software_lib(&self, client_id: i32, software_lib: SoftwareLibrary) -> Result<()> {
        let mut conn = self.pool.get()?;
        let sl: Vec<SoftwareVersion> = software_lib
            .software
            .iter()
            .map(|e| self.get_software_entry(&e.name, &e.version, e.publisher.clone()))
            .filter_map(|r: Result<SoftwareVersion>| r.ok())
            .collect();
        conn.transaction::<(), diesel::result::Error, _>(|c| {
            diesel::delete(software_list::table)
                .filter(software_list::client_id.eq(client_id))
                .execute(c)?;
            for s in sl {
                diesel::insert_into(software_list::table)
                    .values(NewSoftwareList {
                        client_id: &client_id,
                        software_id: &s.id,
                    })
                    .execute(c)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    pub fn update_profiles(&self, client_id: i32, profiles: UserProfiles) -> Result<()> {
        let mut conn = self.pool.get()?;
        conn.transaction::<(), diesel::result::Error, _>(|c| {
            for p in profiles.profiles {
                let user: User = diesel::insert_into(user::table)
                    .values(NewUser {
                        sid: &p.sid,
                        username: p.username.as_ref(),
                    })
                    .on_conflict(user::sid)
                    .do_update()
                    .set(user::username.eq(p.username.as_ref()))
                    .get_result(c)?;
                if p.size.is_some() {
                    diesel::insert_into(userprofile::table)
                        .values(NewUserProfileWithSize {
                            client_id: &client_id,
                            user_id: &user.id,
                            health_status: &(p.health_status as i16),
                            roaming_configured: &p.roaming_configured,
                            roaming_path: p.roaming_path.as_ref(),
                            roaming_preference: p.roaming_preference.as_ref(),
                            last_use_time: &p.last_use_time.naive_utc(),
                            last_download_time: p.last_download_time.map(|t| t.naive_utc()),
                            last_upload_time: p.last_upload_time.map(|t| t.naive_utc()),
                            status: &(p.status as i64),
                            size: p.size.map(BigDecimal::from),
                        })
                        .on_conflict((userprofile::client_id, userprofile::user_id))
                        .do_update()
                        .set((
                            userprofile::health_status.eq(&(p.health_status as i16)),
                            userprofile::roaming_configured.eq(&p.roaming_configured),
                            userprofile::roaming_path.eq(p.roaming_path.as_ref()),
                            userprofile::roaming_preference.eq(p.roaming_preference.as_ref()),
                            userprofile::last_use_time.eq(&p.last_use_time.naive_utc()),
                            userprofile::last_download_time
                                .eq(p.last_download_time.map(|t| t.naive_utc())),
                            userprofile::last_upload_time
                                .eq(p.last_upload_time.map(|t| t.naive_utc())),
                            userprofile::status.eq(&(p.status as i64)),
                            userprofile::size.eq(p.size.map(BigDecimal::from)),
                        ))
                        .execute(c)?;
                } else {
                    diesel::insert_into(userprofile::table)
                        .values(NewUserProfileWithoutSize {
                            client_id: &client_id,
                            user_id: &user.id,
                            health_status: &(p.health_status as i16),
                            roaming_configured: &p.roaming_configured,
                            roaming_path: p.roaming_path.as_ref(),
                            roaming_preference: p.roaming_preference.as_ref(),
                            last_use_time: &p.last_use_time.naive_utc(),
                            last_download_time: p.last_download_time.map(|t| t.naive_utc()),
                            last_upload_time: p.last_upload_time.map(|t| t.naive_utc()),
                            status: &(p.status as i64),
                        })
                        .on_conflict((userprofile::client_id, userprofile::user_id))
                        .do_update()
                        .set((
                            userprofile::health_status.eq(&(p.health_status as i16)),
                            userprofile::roaming_configured.eq(&p.roaming_configured),
                            userprofile::roaming_path.eq(p.roaming_path.as_ref()),
                            userprofile::roaming_preference.eq(p.roaming_preference.as_ref()),
                            userprofile::last_use_time.eq(&p.last_use_time.naive_utc()),
                            userprofile::last_download_time
                                .eq(p.last_download_time.map(|t| t.naive_utc())),
                            userprofile::last_upload_time
                                .eq(p.last_upload_time.map(|t| t.naive_utc())),
                            userprofile::status.eq(&(p.status as i64)),
                        ))
                        .execute(c)?;
                }
            }
            Ok(())
        })?;
        Ok(())
    }

    fn get_software_entry(
        &self,
        name: &String,
        version: &String,
        publisher: Option<String>,
    ) -> Result<SoftwareVersion> {
        let publisher = &publisher.unwrap_or("".to_string());
        let mut conn = self.pool.get()?;
        let entries: Option<SoftwareInfo> = software_info::table
            .filter(software_info::name.eq(name))
            .filter(software_info::publisher.eq(publisher))
            .first::<SoftwareInfo>(&mut conn)
            .optional()?;
        let software_info = if let Some(entry) = entries {
            Ok(entry)
        } else {
            self.create_software_entry(name, publisher)
        };
        if let Ok(software_info) = software_info {
            let software_version: Option<SoftwareVersion> = software_version::table
                .filter(software_version::software_id.eq(software_info.id))
                .filter(software_version::version.eq(version))
                .first(&mut conn)
                .optional()?;
            if let Some(software_version) = software_version {
                Ok(software_version)
            } else {
                self.create_software_version(software_info.id, version)
            }
        } else {
            Err(software_info.unwrap_err())
        }
    }

    fn create_software_entry(&self, name: &String, publisher: &String) -> Result<SoftwareInfo> {
        let mut conn = self.pool.get()?;
        Ok(diesel::insert_into(software_info::table)
            .values(NewSoftwareInfo {
                name,
                publisher: Some(publisher),
            })
            .get_result(&mut conn)?)
    }

    fn create_software_version(
        &self,
        software_id: i32,
        version: &String,
    ) -> Result<SoftwareVersion> {
        let mut conn = self.pool.get()?;
        Ok(diesel::insert_into(software_version::table)
            .values(NewSoftwareVersion {
                software_id: &software_id,
                version,
            })
            .get_result(&mut conn)?)
    }

    pub fn get_clients_with_os_info(&self) -> Result<Vec<(Client, Option<OsInfo>)>> {
        let mut conn = self.pool.get()?;
        Ok(client::table
            .left_join(os_info::table)
            .order_by(os_info::computer_name)
            .load::<(Client, Option<OsInfo>)>(&mut conn)?)
    }

    pub fn get_software_list(&self) -> Result<Vec<SoftwareInfo>> {
        let mut conn = self.pool.get()?;
        Ok(software_info::table
            .order_by(software_info::name)
            .load::<SoftwareInfo>(&mut conn)?)
    }

    pub fn get_software_info(&self, software_id: i32) -> Result<SoftwareInfo> {
        let mut conn = self.pool.get()?;
        Ok(software_info::table
            .filter(software_info::id.eq(software_id))
            .get_result::<SoftwareInfo>(&mut conn)?)
    }

    pub fn get_software_versions(&self, software_id: i32) -> Result<Vec<SoftwareVersionWithCount>> {
        let mut conn = self.pool.get()?;
        Ok(software_version::table
            .select((
                software_version::id,
                software_version::software_id,
                software_version::version,
                coalesce(
                    software_list::table
                        .filter(software_list::software_id.eq(software_version::id))
                        .count()
                        .single_value(),
                    0,
                ),
            ))
            .filter(software_version::software_id.eq(software_id))
            .load::<SoftwareVersionWithCount>(&mut conn)?)
    }

    pub fn get_software_computer_list(
        &self,
        software_id: i32,
    ) -> Result<Vec<(SoftwareList, SoftwareVersion, (Client, OsInfo))>> {
        let mut conn = self.pool.get()?;
        Ok(software_list::table
            .filter(software_version::software_id.eq(software_id))
            .inner_join(software_version::table)
            .inner_join(client::table.inner_join(os_info::table))
            .load::<(SoftwareList, SoftwareVersion, (Client, OsInfo))>(&mut conn)?)
    }

    pub fn get_software_version(&self, version_id: i32) -> Result<SoftwareVersion> {
        let mut conn = self.pool.get()?;
        Ok(software_version::table
            .filter(software_version::id.eq(version_id))
            .get_result::<SoftwareVersion>(&mut conn)?)
    }

    pub fn get_software_version_clients(&self, version_id: i32) -> Result<Vec<(Client, OsInfo)>> {
        let mut conn = self.pool.get()?;
        Ok(client::table
            .filter(
                client::id.eq_any(
                    software_list::table
                        .select(software_list::client_id)
                        .filter(software_list::software_id.eq(version_id)),
                ),
            )
            .inner_join(os_info::table)
            .load::<(Client, OsInfo)>(&mut conn)?)
    }

    pub fn get_client_os_info(&self, uuid: &Uuid) -> Result<OsInfo> {
        let mut conn = self.pool.get()?;
        Ok(os_info::table
            .filter(
                os_info::client_id.nullable().eq(client::table
                    .select(client::id)
                    .filter(client::uuid.eq(uuid))
                    .single_value()),
            )
            .get_result::<OsInfo>(&mut conn)?)
    }

    pub fn get_client_profiles(&self, uuid: &Uuid) -> Result<Vec<(UserProfile, User)>> {
        let mut conn = self.pool.get()?;
        Ok(userprofile::table
            .filter(
                userprofile::client_id.nullable().eq(client::table
                    .select(client::id)
                    .filter(client::uuid.eq(uuid))
                    .single_value()),
            )
            .inner_join(user::table)
            .order_by(user::username)
            .load::<(UserProfile, User)>(&mut conn)?)
    }

    pub fn get_client_software(
        &self,
        uuid: Uuid,
    ) -> Result<Vec<(SoftwareList, Client, (SoftwareVersion, SoftwareInfo))>> {
        let mut conn = self.pool.get()?;
        let software_version_list: Vec<(SoftwareList, Client, (SoftwareVersion, SoftwareInfo))> =
            software_list::table
                .filter(client::uuid.eq(uuid))
                .inner_join(client::table)
                .inner_join(software_version::table.inner_join(software_info::table))
                .order_by(software_info::name)
                .load::<(SoftwareList, Client, (SoftwareVersion, SoftwareInfo))>(&mut conn)?;
        Ok(software_version_list)
    }

    pub fn get_profiles(&self) -> Result<Vec<UserWithProfileCount>> {
        let mut conn = self.pool.get()?;
        Ok(user::table
            .select((
                user::id,
                user::sid,
                user::username,
                coalesce(
                    userprofile::table
                        .filter(userprofile::user_id.eq(user::id))
                        .count()
                        .single_value(),
                    0,
                ),
            ))
            .order_by(user::username)
            .load::<UserWithProfileCount>(&mut conn)?)
    }

    pub fn get_profile_info(
        &self,
        sid: String,
    ) -> Result<Vec<(UserProfile, User, Client, Option<OsInfo>)>> {
        let mut conn = self.pool.get()?;
        Ok(userprofile::table
            .filter(user::sid.eq(sid))
            .inner_join(user::table)
            .inner_join(client::table)
            .left_join(os_info::table.on(os_info::client_id.eq(userprofile::client_id)))
            .load::<(UserProfile, User, Client, Option<OsInfo>)>(&mut conn)?)
    }

    pub fn get_processors_count(&self) -> Result<Vec<ProcessorCount>> {
        let mut conn = self.pool.get()?;
        Ok(processor::table
            .group_by((processor::name, processor::manufacturer))
            .select((
                processor::name,
                processor::manufacturer,
                max(processor::cores),
                max(processor::logical_cores),
                max(processor::clock_speed),
                max(processor::address_width),
                count(processor::name),
            ))
            .order_by(processor::name)
            .load::<ProcessorCount>(&mut conn)?)
    }

    pub fn get_processor_clients(&self, processor: &String) -> Result<Vec<(Client, OsInfo)>> {
        let mut conn = self.pool.get()?;
        Ok(client::table
            .filter(
                client::id.eq_any(
                    processor::table
                        .select(processor::client_id)
                        .filter(processor::name.eq(processor)),
                ),
            )
            .inner_join(os_info::table)
            .order_by(os_info::computer_name)
            .load::<(Client, OsInfo)>(&mut conn)?)
    }

    pub fn get_client_processors(&self, uuid: Uuid) -> Result<Vec<Processor>> {
        let mut conn = self.pool.get()?;
        Ok(processor::table
            .filter(
                processor::client_id.nullable().eq(client::table
                    .select(client::id)
                    .filter(client::uuid.eq(uuid))
                    .single_value()),
            )
            .load::<Processor>(&mut conn)?)
    }

    pub fn get_memorys_count(&self) -> Result<Vec<MemoryCount>> {
        let mut conn = self.pool.get()?;
        Ok(diesel::sql_query(
            "SELECT capacity, sticks, COUNT(*) FROM memory GROUP BY capacity, sticks ORDER BY capacity, sticks;",
        )
        .load(&mut conn)?)
    }

    pub fn get_memory_clients(&self, size: u64, stick_count: i64) -> Result<Vec<(Client, OsInfo)>> {
        let mut conn = self.pool.get()?;
        Ok(client::table
            .filter(
                client::id.eq_any(
                    memory_stick::table
                        .group_by(memory_stick::client_id)
                        .select(memory_stick::client_id)
                        .having(
                            sum(memory_stick::capacity)
                                .eq(BigDecimal::from(size))
                                .and(count(memory_stick::capacity).eq(stick_count)),
                        ),
                ),
            )
            .inner_join(os_info::table)
            .order_by(os_info::computer_name)
            .load::<(Client, OsInfo)>(&mut conn)?)
    }

    pub fn get_client_memory(&self, uuid: Uuid) -> Result<Vec<Memory>> {
        let mut conn = self.pool.get()?;
        Ok(memory_stick::table
            .group_by(memory_stick::client_id)
            .select((
                memory_stick::client_id,
                sum(memory_stick::capacity),
                count(memory_stick::capacity),
            ))
            .filter(
                memory_stick::client_id.nullable().eq(client::table
                    .select(client::id)
                    .filter(client::uuid.eq(uuid))
                    .single_value()),
            )
            .load::<Memory>(&mut conn)?)
    }

    pub fn get_client_memory_sticks(&self, uuid: Uuid) -> Result<Vec<MemoryStick>> {
        let mut conn = self.pool.get()?;
        Ok(memory_stick::table
            .filter(
                memory_stick::client_id.nullable().eq(client::table
                    .select(client::id)
                    .filter(client::uuid.eq(uuid))
                    .single_value()),
            )
            .load::<MemoryStick>(&mut conn)?)
    }

    pub fn get_graphics_cards_count(&self) -> Result<Vec<GraphicsCardCount>> {
        let mut conn = self.pool.get()?;
        Ok(graphics_card::table
            .group_by(graphics_card::name)
            .select((graphics_card::name, count_star()))
            .order_by(graphics_card::name)
            .load::<GraphicsCardCount>(&mut conn)?)
    }

    pub fn get_graphics_card_clients(&self, card: &String) -> Result<Vec<(Client, OsInfo)>> {
        let mut conn = self.pool.get()?;
        Ok(client::table
            .filter(
                client::id.eq_any(
                    graphics_card::table
                        .select(graphics_card::client_id)
                        .filter(graphics_card::name.eq(card)),
                ),
            )
            .inner_join(os_info::table)
            .order_by(os_info::computer_name)
            .load::<(Client, OsInfo)>(&mut conn)?)
    }

    pub fn get_client_graphics_cards(&self, uuid: Uuid) -> Result<Vec<GraphicsCard>> {
        let mut conn = self.pool.get()?;
        Ok(graphics_card::table
            .filter(
                graphics_card::client_id.nullable().eq(client::table
                    .select(client::id)
                    .filter(client::uuid.eq(uuid))
                    .single_value()),
            )
            .load::<GraphicsCard>(&mut conn)?)
    }

    pub fn get_disks_count(&self) -> Result<Vec<DiskCount>> {
        let mut conn = self.pool.get()?;
        Ok(disks::table
            .group_by((disks::model, disks::size))
            .select((disks::model, disks::size, count_star()))
            .order_by(disks::model)
            .load::<DiskCount>(&mut conn)?)
    }

    pub fn get_disk_clients(&self, model: &String, size: u64) -> Result<Vec<(Client, OsInfo)>> {
        let mut conn = self.pool.get()?;
        Ok(client::table
            .filter(
                client::id.eq_any(
                    disks::table
                        .select(disks::client_id)
                        .filter(disks::model.eq(model))
                        .filter(disks::size.eq(BigDecimal::from(size))),
                ),
            )
            .inner_join(os_info::table)
            .order_by(os_info::computer_name)
            .load::<(Client, OsInfo)>(&mut conn)?)
    }

    pub fn get_client_disks(&self, uuid: Uuid) -> Result<Vec<Disk>> {
        let mut conn = self.pool.get()?;
        Ok(disks::table
            .filter(
                disks::client_id.nullable().eq(client::table
                    .select(client::id)
                    .filter(client::uuid.eq(uuid))
                    .single_value()),
            )
            .load::<Disk>(&mut conn)?)
    }

    pub fn get_computer_models_count(&self) -> Result<Vec<ComputerModelCount>> {
        let mut conn = self.pool.get()?;
        Ok(computer_model::table
            .group_by((computer_model::model_family, computer_model::manufacturer))
            .select((
                computer_model::manufacturer,
                computer_model::model_family,
                count_star(),
            ))
            .order_by((computer_model::manufacturer, computer_model::model_family))
            .load::<ComputerModelCount>(&mut conn)?)
    }

    pub fn get_computer_model_clients(&self, model: &String, manufacturer: &String) -> Result<Vec<(Client, OsInfo)>> {
        let mut conn = self.pool.get()?;
        Ok(client::table
            .filter(
                client::id.eq_any(
                    computer_model::table
                        .select(computer_model::client_id)
                        .filter(computer_model::model_family.eq(model))
                        .filter(computer_model::manufacturer.eq(manufacturer)),
                ),
            )
            .inner_join(os_info::table)
            .order_by(os_info::computer_name)
            .load::<(Client, OsInfo)>(&mut conn)?)
    }

    pub fn get_client_computer_model(&self, uuid: Uuid) -> Result<Vec<ComputerModel>> {
        let mut conn = self.pool.get()?;
        Ok(computer_model::table
            .filter(
                computer_model::client_id.nullable().eq(client::table
                    .select(client::id)
                    .filter(client::uuid.eq(uuid))
                    .single_value()),
            )
            .load::<ComputerModel>(&mut conn)?)
    }

    pub fn get_client_bios(&self, uuid: Uuid) -> Result<Vec<Bios>> {
        let mut conn = self.pool.get()?;
        Ok(bios::table
            .filter(
                bios::client_id.nullable().eq(client::table
                    .select(client::id)
                    .filter(client::uuid.eq(uuid))
                    .single_value()),
            )
            .load::<Bios>(&mut conn)?)
    }

    pub fn get_network_adapters_count(&self) -> Result<Vec<NetworkAdapterCount>> {
        let mut conn = self.pool.get()?;
        Ok(network_adapter::table
            .group_by(network_adapter::name)
            .select((network_adapter::name, count_star()))
            .order_by(network_adapter::name)
            .load::<NetworkAdapterCount>(&mut conn)?)
    }

    pub fn get_network_adapter_clients(&self, name: &String) -> Result<Vec<(Client, OsInfo)>> {
        let mut conn = self.pool.get()?;
        Ok(client::table
            .filter(
                client::id.eq_any(
                    network_adapter::table
                        .select(network_adapter::client_id)
                        .filter(network_adapter::name.eq(name)),
                ),
            )
            .inner_join(os_info::table)
            .order_by(os_info::computer_name)
            .load::<(Client, OsInfo)>(&mut conn)?)
    }

    pub fn get_client_network_adapters(&self, uuid: Uuid) -> Result<Vec<NetworkAdapter>> {
        let mut conn = self.pool.get()?;
        Ok(network_adapter::table
            .filter(
                network_adapter::client_id.nullable().eq(client::table
                    .select(client::id)
                    .filter(client::uuid.eq(uuid))
                    .single_value()),
            )
            .load::<NetworkAdapter>(&mut conn)?)
    }
}
