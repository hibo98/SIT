use rocket::{
    form::Form,
    http::CookieJar,
    request::FlashMessage,
    response::{Flash, Redirect},
    Route, State,
};
use rocket_dyn_templates::{context, Template};

use crate::{auth::User, database::Database};

#[derive(FromForm)]
struct Login<'r> {
    username: &'r str,
    password: &'r str,
}

#[get("/")]
fn index(user: User) -> Template {
    Template::render("auth/index", context! { user_id: user.user_id })
}

#[get("/", rank = 2)]
fn no_auth_index() -> Redirect {
    Redirect::to(uri!("/auth", login_page))
}

#[get("/login")]
fn login(_user: User) -> Redirect {
    Redirect::to(uri!("/"))
}

#[get("/login", rank = 2)]
fn login_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render("auth/login", flash)
}

#[post("/login", data = "<login>")]
fn post_login(db: &State<Database>, jar: &CookieJar<'_>, login: Form<Login<'_>>) -> Redirect {
    let result = crate::auth::login(db, login.username, login.password, jar);
    if result.is_err() {
        Redirect::to(uri!("/auth", login_page))
        // TODO: Add error cause "Invalid username/password."
    } else {
        Redirect::to(uri!("/"))
    }
}

#[get("/logout")]
fn logout(db: &State<Database>, jar: &CookieJar<'_>) -> Flash<Redirect> {
    let _result = crate::auth::logout(db, jar);
    Flash::success(
        Redirect::to(uri!("/auth", login_page)),
        "Successfully logged out.",
    )
}

pub fn routes() -> Vec<Route> {
    routes![
        index,
        no_auth_index,
        login,
        login_page,
        post_login,
        logout,
    ]
}
