#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_dyn_templates;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    SqliteConnection,
};
use rocket::{
    fairing::AdHoc,
    fs::{relative, FileServer},
    routes, Build, Config, Rocket,
};
use rocket_dyn_templates::Template;
use std::env;

pub mod controllers;
use controllers::{
    features::{features, patch_feature},
    health_check,
    shortcuts::{delete_shortcut, get_shortcut, index, put_shortcut},
    teams::list_teams,
    users::{login, simple_login},
};
pub mod guards;
pub mod models;
use crate::models::users::Sessions;
pub mod schema;
use dotenv::dotenv;

embed_migrations!("migrations");

pub struct AppConfig {
    pub simple_login_salt1: String,
    pub simple_login_salt2: String,
}

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConn = PooledConnection<ConnectionManager<SqliteConnection>>;

pub fn server(
    port: u16,
    address: &str,
    db_url: &str,
    sessions: Sessions,
    config: AppConfig,
    run_migration: bool,
    cli_colors: bool,
) -> Rocket<Build> {
    dotenv().ok();

    let db_manager: ConnectionManager<SqliteConnection> = ConnectionManager::new(db_url);
    let db_pool = Pool::builder().max_size(15).build(db_manager).unwrap();

    if run_migration {
        embedded_migrations::run(&db_pool.get().unwrap()).unwrap();
    }

    let rocket_config = Config {
        port,
        address: address.parse().unwrap(),
        cli_colors,
        ..Config::debug_default()
    };

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
                health_check,
                list_teams
            ],
        )
        .mount("/public", FileServer::from(relative!("public")))
        .manage(features)
        .manage(sessions)
        .manage(config)
        .manage(db_pool)
        .attach(Template::fairing())
        .attach(AdHoc::on_response("HTTP code", |_, res| {
            Box::pin(async move {
                if (200..399).contains(&res.status().code) {
                    info!("   >> {}", res.status());
                } else {
                    error!("   >> {}", res.status());
                }
            })
        }))
}
