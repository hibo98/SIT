use std::collections::HashMap;
use std::sync::Mutex;

use super::{model::*, schema::*};
use anyhow::Result;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_types::{BigInt, Nullable};
use sit_lib::os::{ProfileInfo, UserProfiles};
use uuid::Uuid;

sql_function! { fn coalesce(x: Nullable<BigInt>, y: BigInt) -> BigInt; }

pub struct UserManager {
    user_id_cache: Mutex<HashMap<String, i32>>,
    sid_cache: Mutex<HashMap<i32, String>>,
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl UserManager {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> UserManager {
        UserManager {
            user_id_cache: Mutex::new(HashMap::new()),
            sid_cache: Mutex::new(HashMap::new()),
            pool,
        }
    }

    pub fn get_user_id_for_sid(&self, sid: &String) -> Result<Option<i32>> {
        let mut c = self.pool.get()?;
        let mut user_id_cache = self.user_id_cache.lock().unwrap();
        match user_id_cache.get(sid) {
            Some(user_id) => Ok(Some(user_id.to_owned())),
            None => {
                match user::table
                    .filter(user::sid.eq(&sid))
                    .first::<User>(&mut c)
                    .optional()?
                {
                    Some(db_user) => {
                        let user_id: i32 = db_user.id;
                        user_id_cache.insert(sid.clone(), user_id);
                        self.sid_cache.lock().unwrap().insert(user_id, sid.clone());
                        Ok(Some(user_id))
                    }
                    None => Ok(None),
                }
            }
        }
    }

    pub fn get_sid_for_user_id(&self, user_id: i32) -> Result<Option<String>> {
        let mut c = self.pool.get()?;
        let mut sid_cache = self.sid_cache.lock().unwrap();
        match sid_cache.get(&user_id) {
            Some(sid) => Ok(Some(sid.clone())),
            None => {
                match user::table
                    .filter(user::id.eq(&user_id))
                    .first::<User>(&mut c)
                    .optional()?
                {
                    Some(db_user) => {
                        let sid: String = db_user.sid;
                        self.user_id_cache
                            .lock()
                            .unwrap()
                            .insert(sid.clone(), user_id);
                        sid_cache.insert(user_id, sid.clone());
                        Ok(Some(sid))
                    }
                    None => Ok(None),
                }
            }
        }
    }

    pub fn get_user(&self, sid: &String) -> Result<User> {
        let mut conn = self.pool.get()?;
        Ok(user::table
            .filter(user::sid.eq(sid))
            .get_result(&mut conn)?)
    }

    pub fn get_profile_paths(&self, uuid: &Uuid, sid: &String) -> Result<Vec<UserProfilePaths>> {
        let mut conn = self.pool.get()?;
        Ok(userprofile_paths::table
            .filter(
                userprofile_paths::client_id.nullable().eq(client::table
                    .select(client::id)
                    .filter(client::uuid.eq(uuid))
                    .single_value()),
            )
            .filter(
                userprofile_paths::user_id.nullable().eq(user::table
                    .select(user::id)
                    .filter(user::sid.eq(sid))
                    .single_value()),
            )
            .order_by(userprofile_paths::path)
            .load::<UserProfilePaths>(&mut conn)?)
    }

    pub fn get_profiles(&self) -> Result<Vec<UserWithProfileCount>> {
        let mut conn = self.pool.get()?;
        Ok(user::table
            .select((
                user::id,
                user::sid,
                user::username,
                user::domain,
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
        user_id: i32,
    ) -> Result<Vec<(UserProfile, Client, Option<OsInfo>)>> {
        let mut conn = self.pool.get()?;
        Ok(userprofile::table
            .filter(userprofile::user_id.eq(user_id))
            .inner_join(client::table)
            .left_join(os_info::table.on(os_info::client_id.eq(userprofile::client_id)))
            .load::<(UserProfile, Client, Option<OsInfo>)>(&mut conn)?)
    }

    pub fn update_profiles(&self, client_id: i32, profiles: UserProfiles) -> Result<()> {
        let mut conn = self.pool.get()?;
        conn.transaction::<(), anyhow::Error, _>(|c| {
            let existing: Vec<UserProfile> = userprofile::table
                .filter(userprofile::client_id.eq(client_id))
                .load::<UserProfile>(c)?;
            let mut to_add: Vec<(i32, &ProfileInfo)> = vec![];
            let mut to_update: Vec<(i32, &ProfileInfo)> = vec![];
            let mut to_delete: Vec<i32> = vec![];

            for p in &profiles.profiles {
                let user_info = if let (Some(username), Some(domain)) = (&p.username, &p.domain) {
                    Some((username.to_string(), domain.to_string()))
                } else if let Some(username) = &p.username {
                    let mut split = username.split('\\');
                    let d = split.next();
                    let u = split.next();
                    if let (Some(d), Some(u)) = (d, u) {
                        Some((u.to_string(), d.to_string()))
                    } else {
                        None
                    }
                } else {
                    None
                };
                let user_id = match self.get_user_id_for_sid(&p.sid)? {
                    Some(user_id) => {
                        if let Some((username, domain)) = user_info {
                            diesel::update(user::table)
                                .set((user::username.eq(username), user::domain.eq(domain)))
                                .filter(user::id.eq(user_id))
                                .execute(c)?;
                        }
                        user_id
                    }
                    None => {
                        let user: User = if let Some((username, domain)) = user_info {
                            diesel::insert_into(user::table)
                                .values(NewUser {
                                    sid: &p.sid,
                                    username: Some(&username),
                                    domain: Some(&domain),
                                })
                                .get_result(c)?
                        } else {
                            diesel::insert_into(user::table)
                                .values(NewUser {
                                    sid: &p.sid,
                                    username: p.username.as_ref(),
                                    domain: p.domain.as_ref(),
                                })
                                .get_result(c)?
                        };
                        self.user_id_cache
                            .lock()
                            .unwrap()
                            .insert(user.sid.clone(), user.id);
                        self.sid_cache
                            .lock()
                            .unwrap()
                            .insert(user.id, user.sid.clone());
                        user.id
                    }
                };

                if existing.iter().any(|i| i.user_id.eq(&user_id)) {
                    to_update.push((user_id, p));
                } else {
                    to_add.push((user_id, p));
                }
            }

            for up in existing {
                if !profiles.profiles.iter().any(|i| {
                    if let Ok(Some(sid)) = self.get_sid_for_user_id(up.user_id) {
                        sid.eq(&i.sid)
                    } else {
                        false
                    }
                }) {
                    to_delete.push(up.user_id);
                }
            }

            for (user_id, p) in to_add {
                if p.size.is_some() {
                    diesel::insert_into(userprofile::table)
                        .values(NewUserProfileWithSize {
                            client_id: &client_id,
                            user_id: &user_id,
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
                        .execute(c)?;
                } else {
                    diesel::insert_into(userprofile::table)
                        .values(NewUserProfileWithoutSize {
                            client_id: &client_id,
                            user_id: &user_id,
                            health_status: &(p.health_status as i16),
                            roaming_configured: &p.roaming_configured,
                            roaming_path: p.roaming_path.as_ref(),
                            roaming_preference: p.roaming_preference.as_ref(),
                            last_use_time: &p.last_use_time.naive_utc(),
                            last_download_time: p.last_download_time.map(|t| t.naive_utc()),
                            last_upload_time: p.last_upload_time.map(|t| t.naive_utc()),
                            status: &(p.status as i64),
                        })
                        .execute(c)?;
                }
                if let Some(path_size) = p.path_size.as_ref() {
                    for p in path_size {
                        let path: Result<UserProfilePaths, _> = userprofile_paths::table
                            .filter(userprofile_paths::client_id.eq(&client_id))
                            .filter(userprofile_paths::user_id.eq(&user_id))
                            .filter(userprofile_paths::path.eq(&p.path))
                            .get_result(c);

                        if let Ok(path) = path {
                            diesel::update(userprofile_paths::table)
                                .set(userprofile_paths::size.eq(BigDecimal::from(p.size)))
                                .filter(userprofile_paths::id.eq(path.id))
                                .execute(c)?;
                        } else {
                            diesel::insert_into(userprofile_paths::table)
                                .values(NewUserProfilePaths {
                                    client_id: &client_id,
                                    user_id: &user_id,
                                    path: &p.path,
                                    size: BigDecimal::from(p.size),
                                })
                                .execute(c)?;
                        }
                    }
                }
            }

            for user_id in to_delete {
                diesel::delete(userprofile::table)
                    .filter(userprofile::client_id.eq(client_id))
                    .filter(userprofile::user_id.eq(user_id))
                    .execute(c)?;
            }

            for (user_id, p) in to_update {
                if p.size.is_some() {
                    diesel::update(userprofile::table)
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
                        .filter(userprofile::client_id.eq(client_id))
                        .filter(userprofile::user_id.eq(user_id))
                        .execute(c)?;
                } else {
                    diesel::update(userprofile::table)
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
                        .filter(userprofile::client_id.eq(client_id))
                        .filter(userprofile::user_id.eq(user_id))
                        .execute(c)?;
                }
                if let Some(path_size) = p.path_size.as_ref() {
                    for p in path_size {
                        let path: Result<UserProfilePaths, _> = userprofile_paths::table
                            .filter(userprofile_paths::client_id.eq(&client_id))
                            .filter(userprofile_paths::user_id.eq(&user_id))
                            .filter(userprofile_paths::path.eq(&p.path))
                            .get_result(c);

                        if let Ok(path) = path {
                            diesel::update(userprofile_paths::table)
                                .set(userprofile_paths::size.eq(BigDecimal::from(p.size)))
                                .filter(userprofile_paths::id.eq(path.id))
                                .execute(c)?;
                        } else {
                            diesel::insert_into(userprofile_paths::table)
                                .values(NewUserProfilePaths {
                                    client_id: &client_id,
                                    user_id: &user_id,
                                    path: &p.path,
                                    size: BigDecimal::from(p.size),
                                })
                                .execute(c)?;
                        }
                    }
                }
            }
            Ok(())
        })?;
        Ok(())
    }

    pub fn _invalidate_cache(&self) {
        self.user_id_cache.lock().unwrap().clear();
        self.sid_cache.lock().unwrap().clear();
    }
}
