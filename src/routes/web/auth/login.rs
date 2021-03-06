use config::Config;
use database::DbConn;
use database::models::users::User;
use database::schema::users;
use errors::*;
use routes::web::{Rst, OptionalWebUser, Session};

use diesel::prelude::*;

use rocket::State;
use rocket::http::{Cookies, Cookie};
use rocket::request::Form;
use rocket::response::Redirect;

use rocket_contrib::Template;

#[get("/login")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Rst {
  if user.is_some() {
    return Rst::Redirect(Redirect::to("/"));
  }
  let ctx = json!({
    "config": &*config,
    // TODO: this can be made into an optional request guard
    "error": sess.data.remove("error"),
    "server_version": ::SERVER_VERSION,
    "resources_version": &*::RESOURCES_VERSION,
  });
  Rst::Template(Template::render("auth/login", ctx))
}

#[derive(Debug, FromForm)]
struct RegistrationData {
  username: String,
  password: String,
}

#[post("/login", format = "application/x-www-form-urlencoded", data = "<data>")]
fn post(data: Form<RegistrationData>, mut sess: Session, mut cookies: Cookies, conn: DbConn) -> Result<Redirect> {
  let data = data.into_inner();

  let user: Option<User> = users::table
    .filter(users::username.eq(&data.username))
    .first(&*conn)
    .optional()?;

  let user = match user {
    Some(u) => u,
    None => {
      sess.data.insert("error".into(), "Username not found.".into());
      return Ok(Redirect::to("/login"));
    },
  };

  if !user.check_password(&data.password) {
    sess.data.insert("error".into(), "Incorrect password.".into());
    return Ok(Redirect::to("/login"));
  }

  cookies.add_private(Cookie::new("user_id", user.id().simple().to_string()));

  Ok(Redirect::to("lastpage"))
}
