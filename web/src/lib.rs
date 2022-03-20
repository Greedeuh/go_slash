#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_dyn_templates;
#[macro_use]
extern crate diesel;
use rocket::figment::{
    util::map,
    value::{Map, Value},
};
use rocket::{
    fs::{relative, FileServer},
    routes, Build, Config, Rocket,
};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::database;
use std::env;

pub mod controllers;
use controllers::{
    features::{features, patch_feature},
    health_check,
    shortcuts::{delete_shortcut, get_shortcut, index, put_shortcut},
    users::{login, simple_login},
};
pub mod models;
pub use models::{features::GlobalFeatures, shortcuts::Entries, users::SimpleUsers};
pub mod guards;
use crate::models::users::Sessions;
mod schema;
use dotenv::dotenv;

pub struct AppConfig {
    pub simple_login_salt1: String,
    pub simple_login_salt2: String,
}

pub fn server(
    port: u16,
    address: &str,
    db_url: &str,
    entries: Entries,
    features: GlobalFeatures,
    users: SimpleUsers,
    sessions: Sessions,
    config: AppConfig,
) -> Rocket<Build> {
    dotenv().ok();

    let db_config: Map<_, Value> = map! {
        "url" => db_url.into(),
        "pool_size" => 10i16.into()
    };

    let rocket_config = Config {
        port,
        address: address.parse().unwrap(),
        ..Config::debug_default()
    };

    let rocket_config = rocket::Config::figment()
        .merge(("databases", map!["go" => db_config]))
        .merge(rocket_config);

    rocket::build()
        .configure(rocket_config)
        .mount(
            "/",
            routes![
                index,
                get_shortcut,
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
        .attach(DbConn::fairing())
        .attach(Template::fairing())
}

#[database("go")]
struct DbConn(diesel::SqliteConnection);
