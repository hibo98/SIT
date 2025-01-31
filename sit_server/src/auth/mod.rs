use anyhow::{bail, Result};
use argon2::Argon2;
use chrono::{Duration, NaiveDateTime, Utc};
use password_hash::{rand_core::OsRng, PasswordHash, SaltString};
use rand::{distr::Alphanumeric, rng, Rng};
use rocket::{
    http::{private::cookie::Expiration, Cookie, CookieJar, Status},
    outcome::try_outcome,
    request::{FromRequest, Outcome, Request},
    time::OffsetDateTime,
    State,
};
use serde::Serialize;

use crate::database::Database;

const COOKIE_SESSION_ID: &str = "SIT_SESSION";

#[derive(Serialize)]
pub struct User {
    pub user_id: i32,
    pub username: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<User, ()> {
        let db: &State<Database> = try_outcome!(request.guard::<&State<Database>>().await);
        if let Some(mut cookie) = request.cookies().get_private(COOKIE_SESSION_ID) {
            if let Ok(session) = db.get_auth_session_by_session_id(cookie.value()) {
                if Utc::now()
                    .naive_utc()
                    .signed_duration_since(session.valid_until)
                    .gt(&Duration::zero())
                {
                    return Outcome::Forward(Status::SeeOther);
                }
                if let Ok(user) = db.get_auth_user_by_id(session.user_id) {
                    if let Ok((naive, offset)) = calc_current_exp_time() {
                        cookie.set_expires(Expiration::from(offset));
                        request.cookies().add_private(cookie);
                        let _ = db.update_session_exp(&session.session_id, naive);
                    }
                    return Outcome::Success(User {
                        user_id: user.id,
                        username: user.username,
                    });
                }
            }
            Outcome::Forward(Status::SeeOther)
        } else {
            Outcome::Forward(Status::SeeOther)
        }
    }
}

pub fn login(db: &Database, username: &str, password: &str, cookie_jar: &CookieJar) -> Result<()> {
    let user = check_password(db, username, password)?;
    let session_id = get_new_session_id(db);
    let (naive, offset) = calc_current_exp_time()?;
    let mut cookie = Cookie::new(COOKIE_SESSION_ID, session_id.clone());
    cookie.set_expires(Expiration::from(offset));
    cookie_jar.add_private(cookie);
    db.add_new_session(user.user_id, &session_id, naive)?;
    Ok(())
}

pub fn logout(db: &Database, jar: &CookieJar) -> Result<()> {
    if let Some(cookie) = jar.get_private(COOKIE_SESSION_ID) {
        jar.remove_private(Cookie::from(COOKIE_SESSION_ID));
        db.delete_session(cookie.value())?;
    }
    Ok(())
}

pub fn check_password(db: &Database, username: &str, password: &str) -> Result<User> {
    if let Ok(user) = db.get_auth_user_by_username(username) {
        if let Ok(hash) = PasswordHash::new(&user.password) {
            if hash
                .verify_password(&[&Argon2::default()], password)
                .is_ok()
            {
                Ok(User {
                    user_id: user.id,
                    username: user.username,
                })
            } else {
                bail!("Invalid username/password.")
            }
        } else {
            bail!("Internal Server Error")
        }
    } else {
        bail!("Invalid username/password.")
    }
}

pub fn set_new_password(db: &Database, username: &str, new_password: &str) -> Result<()> {
    let user = db.get_auth_user_by_username(username)?;
    let salt = SaltString::generate(OsRng);
    if let Ok(hash) = PasswordHash::generate(Argon2::default(), new_password, &salt) {
        let password_hash_string = hash.to_string();
        db.set_auth_user_password(user.id, &password_hash_string)?;
        Ok(())
    } else {
        bail!("Error generating password hash!")
    }
}

pub fn create_new_user(db: &Database, username: &str, password: &str) -> Result<()> {
    let salt = SaltString::generate(OsRng);
    if let Ok(hash) = PasswordHash::generate(Argon2::default(), password, &salt) {
        let password_hash_string = hash.to_string();
        db.new_auth_user(username, &password_hash_string)?;
        Ok(())
    } else {
        bail!("Error generating password hash!")
    }
}

pub fn delete_user(db: &Database, username: &str) -> Result<()> {
    let user = db.get_auth_user_by_username(username)?;
    db.delete_auth_user(user.id)?;
    Ok(())
}

fn get_new_session_id(db: &Database) -> String {
    loop {
        let session_id = generate_session_id();
        let session = db.get_auth_session_by_session_id(&session_id);
        if session.is_err() {
            return session_id;
        }
    }
}

fn generate_session_id() -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .take(128)
        .map(|c| c as char)
        .collect()
}

fn calc_current_exp_time() -> Result<(NaiveDateTime, OffsetDateTime)> {
    let datetime = Utc::now() + Duration::days(14);
    let offset_datetime = OffsetDateTime::from_unix_timestamp(datetime.timestamp())?;
    Ok((datetime.naive_utc(), offset_datetime))
}
