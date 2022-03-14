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
use serde_json::{json, Value};

pub mod controllers;
use controllers::{
    features::{features, patch_feature},
    shortcuts::{delete_shortcut, put_shortcut, shortcuts},
    users::{login, simple_login},
};
pub mod models;
pub use models::{features::GlobalFeatures, shortcuts::Entries, users::SimpleUsers};
pub mod guards;
use crate::models::users::Sessions;

#[get("/")]
fn index(entries: &State<Entries>) -> Result<Template, (Status, Value)> {
    let all_shortcuts = entries.sorted()?;

    let all_shortcuts = all_shortcuts
        .iter()
        .map(|(shortcut, url)| json!({"shortcut": shortcut, "url": url}))
        .collect::<Vec<_>>();

    let all_shortcuts: String = json!(all_shortcuts).to_string();

    Ok(Template::render(
        "index",
        json!({ "shortcuts": all_shortcuts }),
    ))
}

pub struct AppConfig {
    pub simple_login_salt1: String,
    pub simple_login_salt2: String,
}

pub fn server(
    port: u16,
    entries: Entries,
    features: GlobalFeatures,
    users: SimpleUsers,
    sessions: Sessions,
    config: AppConfig,
) -> Rocket<Build> {
    rocket::build()
        .configure(Config {
            port,
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
