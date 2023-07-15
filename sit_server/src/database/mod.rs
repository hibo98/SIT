extern crate dotenv;

use std::env;

use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::dsl::{count, count_star, max, sum};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_types::{BigInt, Nullable};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use sit_lib::hardware::HardwareInfo;
use sit_lib::licenses::LicenseBundle;
use sit_lib::os::WinOsInfo;
use sit_lib::software::SoftwareLibrary;
use sit_lib::system_status::VolumeList;
use uuid::Uuid;

use crate::database::model::*;
use crate::database::schema::*;

use self::domain_user::UserManager;

mod domain_user;
mod model;
mod schema;

sql_function! { fn coalesce(x: Nullable<BigInt>, y: BigInt) -> BigInt; }

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub struct Database {
    pool: Pool<ConnectionManager<PgConnection>>,
    user_manager: UserManager,
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

        Database {
            pool: pool.clone(),
            user_manager: UserManager::new(pool),
        }
    }

    pub fn user_manager(&self) -> &UserManager {
        &self.user_manager
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

    pub fn update_status_volumes(&self, client_id: i32, volumes: VolumeList) -> Result<()> {
        let mut conn = self.pool.get()?;
        conn.transaction::<(), diesel::result::Error, _>(|c| {
            diesel::delete(volume_status::table)
                .filter(volume_status::client_id.eq(client_id))
                .execute(c)?;
            for v in volumes.volumes {
                diesel::insert_into(volume_status::table)
                    .values(NewVolumeStatus {
                        client_id: &client_id,
                        drive_letter: &v.drive_letter,
                        label: v.label.as_ref(),
                        file_system: &v.file_system,
                        capacity: BigDecimal::from(v.capacity),
                        free_space: BigDecimal::from(v.free_space),
                    })
                    .execute(c)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    pub fn update_license_keys(&self, client_id: i32, license_bundes: LicenseBundle) -> Result<()> {
        self.pool
            .get()?
            .transaction::<(), diesel::result::Error, _>(|conn| {
                let existing: Vec<LicenseKey> = license_key::table
                    .filter(license_key::client_id.eq(client_id))
                    .load::<LicenseKey>(conn)?;
                let mut to_add: Vec<NewLicenseKey> = vec![];
                let mut to_update: Vec<(String, String)> = vec![];
                let mut to_delete: Vec<i32> = vec![];

                for l in &license_bundes.licenses {
                    if !existing.iter().any(|i| i.name.eq(&l.name)) {
                        to_add.push(NewLicenseKey {
                            client_id: &client_id,
                            name: &l.name,
                            key: &l.key,
                        });
                    } else if !existing
                        .iter()
                        .any(|i| i.name.eq(&l.name) && i.key.eq(&l.key))
                    {
                        to_update.push((l.name.clone(), l.key.clone()));
                    }
                }

                for lk in existing {
                    if !&license_bundes.licenses.iter().any(|i| i.name.eq(&lk.name)) {
                        to_delete.push(lk.id);
                    }
                }

                if !to_add.is_empty() {
                    diesel::insert_into(license_key::table)
                        .values(to_add)
                        .execute(conn)?;
                }

                if !to_delete.is_empty() {
                    diesel::delete(license_key::table)
                        .filter(license_key::id.eq_any(to_delete))
                        .execute(conn)?;
                }

                for (name, key) in to_update {
                    diesel::update(license_key::table)
                        .set(license_key::key.eq(key))
                        .filter(license_key::client_id.eq(client_id))
                        .filter(license_key::name.eq(name))
                        .execute(conn)?;
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
            .order_by(software_version::version)
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
            .order_by((software_version::version, os_info::computer_name))
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
            .order_by(os_info::computer_name)
            .load::<(Client, OsInfo)>(&mut conn)?)
    }

    pub fn get_license_list(&self) -> Result<Vec<LicenseKeyCount>> {
        let mut conn = self.pool.get()?;
        Ok(license_key::table
            .group_by(license_key::name)
            .select((license_key::name, count_star()))
            .load::<LicenseKeyCount>(&mut conn)?)
    }

    pub fn get_license_with_computers(
        &self,
        name: &String,
    ) -> Result<Vec<(LicenseKey, (Client, OsInfo))>> {
        let mut conn = self.pool.get()?;
        Ok(license_key::table
            .filter(license_key::name.eq(name))
            .inner_join(client::table.inner_join(os_info::table))
            .order_by(os_info::computer_name)
            .load::<(LicenseKey, (Client, OsInfo))>(&mut conn)?)
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

    pub fn get_computer_model_clients(
        &self,
        model: &String,
        manufacturer: &String,
    ) -> Result<Vec<(Client, OsInfo)>> {
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

    pub fn get_client_licenses(&self, uuid: Uuid) -> Result<Vec<LicenseKey>> {
        let mut conn = self.pool.get()?;
        Ok(license_key::table
            .filter(
                license_key::client_id.nullable().eq(client::table
                    .select(client::id)
                    .filter(client::uuid.eq(uuid))
                    .single_value()),
            )
            .order_by(license_key::name)
            .load::<LicenseKey>(&mut conn)?)
    }

    pub fn get_client_volume_status(&self, uuid: Uuid) -> Result<Vec<VolumeStatus>> {
        let mut conn = self.pool.get()?;
        Ok(volume_status::table
            .filter(
                volume_status::client_id.nullable().eq(client::table
                    .select(client::id)
                    .filter(client::uuid.eq(uuid))
                    .single_value()),
            )
            .order_by(volume_status::drive_letter)
            .load::<VolumeStatus>(&mut conn)?)
    }

    pub fn get_system_status_volume_crit(&self) -> Result<Vec<(VolumeStatus, (Client, OsInfo))>> {
        let mut conn = self.pool.get()?;
        Ok(volume_status::table
            .inner_join(client::table.inner_join(os_info::table))
            .filter(
                (volume_status::free_space / volume_status::capacity)
                    .lt(BigDecimal::try_from(0.1)?),
            )
            .or_filter(volume_status::free_space.lt(BigDecimal::from(5000000000_u64)))
            .load::<(VolumeStatus, (Client, OsInfo))>(&mut conn)?)
    }

    pub fn new_auth_user(&self, username: &str, password_hash: &str) -> Result<AuthUser> {
        let mut conn = self.pool.get()?;
        Ok(diesel::insert_into(auth_user::table)
            .values(NewAuthUser {
                username,
                password: password_hash,
            })
            .get_result(&mut conn)?)
    }

    pub fn get_auth_users(&self) -> Result<Vec<AuthUser>> {
        let mut conn = self.pool.get()?;
        Ok(auth_user::table
            .order_by(auth_user::username)
            .load::<AuthUser>(&mut conn)?)
    }

    pub fn get_auth_user_by_username(&self, username: &str) -> Result<AuthUser> {
        let mut conn = self.pool.get()?;
        Ok(auth_user::table
            .filter(auth_user::username.eq(username))
            .get_result(&mut conn)?)
    }

    pub fn get_auth_user_by_id(&self, id: i32) -> Result<AuthUser> {
        let mut conn = self.pool.get()?;
        Ok(auth_user::table
            .filter(auth_user::id.eq(id))
            .get_result(&mut conn)?)
    }

    pub fn set_auth_user_password(&self, user_id: i32, password_hash: &str) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(diesel::update(auth_user::table)
            .filter(auth_user::id.eq(user_id))
            .set(auth_user::password.eq(password_hash))
            .execute(&mut conn)?)
    }

    pub fn delete_auth_user(&self, user_id: i32) -> Result<()> {
        let mut conn = self.pool.get()?;
        diesel::delete(auth_user::table)
            .filter(auth_user::id.eq(user_id))
            .execute(&mut conn)?;
        Ok(())
    }

    pub fn get_auth_session_by_session_id(&self, session_id: &str) -> Result<AuthSessions> {
        let mut conn = self.pool.get()?;
        Ok(auth_sessions::table
            .filter(auth_sessions::session_id.eq(session_id))
            .get_result(&mut conn)?)
    }

    pub fn update_session_exp(
        &self,
        session_id: &str,
        valid_until: NaiveDateTime,
    ) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(diesel::update(auth_sessions::table)
            .filter(auth_sessions::session_id.eq(session_id))
            .set(auth_sessions::valid_until.eq(valid_until))
            .execute(&mut conn)?)
    }

    pub fn add_new_session(
        &self,
        user_id: i32,
        session_id: &str,
        valid_until: NaiveDateTime,
    ) -> Result<AuthSessions> {
        let mut conn = self.pool.get()?;
        Ok(diesel::insert_into(auth_sessions::table)
            .values(NewAuthSessions {
                user_id: &user_id,
                session_id,
                valid_until,
            })
            .get_result(&mut conn)?)
    }

    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        let mut conn = self.pool.get()?;
        diesel::delete(auth_sessions::table)
            .filter(auth_sessions::session_id.eq(session_id))
            .execute(&mut conn)?;
        Ok(())
    }
}
