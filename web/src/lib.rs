#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_dyn_templates;
use rocket::{
    fs::{relative, FileServer},
    routes, Build, Config, Rocket,
};
use rocket_dyn_templates::Template;

pub mod controllers;
use controllers::{
    features::{features, patch_feature},
    health_check,
    shortcuts::{delete_shortcut, index, put_shortcut, shortcuts},
    users::{login, simple_login},
};
pub mod models;
pub use models::{features::GlobalFeatures, shortcuts::Entries, users::SimpleUsers};
pub mod guards;
use crate::models::users::Sessions;

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
                simple_login,
                health_check
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
