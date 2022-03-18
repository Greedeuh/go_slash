#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_dyn_templates;

use rocket::{
    fs::{relative, FileServer},
    http::Status,
    Build, Config, Rocket, State,
};
use rocket_dyn_templates::Template;
use serde_json::json;

pub mod controllers;
use controllers::{
    features::{features, patch_feature},
    shortcuts::{delete_shortcut, put_shortcut, shortcuts},
    users::{login, simple_login},
};
pub mod models;
pub use models::{features::GlobalFeatures, shortcuts::Entries, users::SimpleUsers};
pub mod guards;
use crate::{
    guards::SessionId,
    models::users::{read_or_write, should_be_logged_in_if_features, Right, Sessions},
};

#[get("/")]
fn index(
    entries: &State<Entries>,
    session_id: Option<SessionId>,
    sessions: &State<Sessions>,
    features: &State<GlobalFeatures>,
) -> Result<Template, (Status, Template)> {
    let user_mail = should_be_logged_in_if_features(&Right::Read, &session_id, sessions, features)?;

    let all_shortcuts = entries.sorted()?;

    let all_shortcuts = all_shortcuts
        .iter()
        .map(|(shortcut, url)| json!({"shortcut": shortcut, "url": url}))
        .collect::<Vec<_>>();

    let all_shortcuts: String = json!(all_shortcuts).to_string();

    let right = read_or_write(features, &user_mail)?;

    Ok(Template::render(
        "index",
        json!({ "shortcuts": all_shortcuts, "right": right, "mail": user_mail }),
    ))
}

pub struct AppConfig {
    pub simple_login_salt1: String,
    pub simple_login_salt2: String,
}

pub fn server(
    port: u16,
    address: &str,
    entries: Entries,
    features: GlobalFeatures,
    users: SimpleUsers,
    sessions: Sessions,
    config: AppConfig,
) -> Rocket<Build> {
    rocket::build()
        .configure(Config {
            port,
            address: address.parse().unwrap(),
            ..Config::debug_default()
        })
        .mount(
            "/",
            routes![
                index,
                shortcuts,
                put_shortcut,
                delete_shortcut,
                login,
                features,
                patch_feature,
                simple_login
            ],
        )
        .mount("/public", FileServer::from(relative!("public")))
        .manage(entries)
        .manage(features)
        .manage(users)
        .manage(sessions)
        .manage(config)
        .attach(Template::fairing())
}
