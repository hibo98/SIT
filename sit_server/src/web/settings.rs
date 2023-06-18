use rocket::{form::Form, response::Redirect, Route, State};
use rocket_dyn_templates::{context, Template};

use crate::{auth::User, database::Database};

#[derive(FromForm)]
struct Login<'r> {
    username: &'r str,
    password: &'r str,
}

#[get("/")]
fn index(user: User) -> Template {
    Template::render("settings/index", context! { user })
}

#[get("/users")]
fn users(db: &State<Database>, user: User) -> Template {
    let users_result = db.get_auth_users();
    if let Ok(auth_users) = users_result {
        Template::render("settings/users", context! { user, auth_users })
    } else {
        Template::render("settings/users", context! { user })
    }

}

#[get("/users/new")]
fn new_user(_user: User) -> Template {
    Template::render("settings/users_new", context! {})
}

#[post("/users/new", data = "<user>")]
fn post_new_user(db: &State<Database>, user: Form<Login<'_>>, _guard: User) -> Redirect {
    let result = crate::auth::create_new_user(db, user.username, user.password);
    if result.is_err() {
        Redirect::to(uri!("/auth", post_new_user))
        // TODO: Add error cause
    } else {
        Redirect::to(uri!("/auth", users))
    }
}

pub fn routes() -> Vec<Route> {
    routes![index, users, new_user, post_new_user,]
}
