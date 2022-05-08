#![cfg_attr(test, feature(proc_macro_hygiene))]
#![feature(let_chains)]
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
    PgConnection,
};
use rocket::{fairing::AdHoc, fs::FileServer, routes, Build, Config, Rocket};
use rocket_dyn_templates::Template;

pub mod controllers;
use controllers::{
    health_check,
    login::{google_login, login, login_redirect_google, simple_login},
    settings::{patch_settings, settings},
    shortcuts::{delete_shortcut, get_shortcut, index, put_shortcut},
    teams::{create_team, delete_team, list_teams, patch_team, show_team},
    users::{
        delete_user_capability, join_global_team, join_team, leave_global_team, leave_team,
        list_users, put_user_capability, put_user_team_ranks,
    },
};
pub mod guards;
pub mod models;
use crate::{models::users::Sessions, services::oidc::OidcService};
pub mod schema;
pub mod services;
mod views;

embed_migrations!("migrations");

pub struct AppConfig {
    pub simple_login_salt1: String,
    pub simple_login_salt2: String,
}

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;

#[allow(clippy::too_many_arguments)]
pub fn server(
    port: u16,
    address: &str,
    db_url: &str,
    sessions: Sessions,
    config: AppConfig,
    run_migration: bool,
    cli_colors: bool,
    oidc_service: OidcService,
) -> Rocket<Build> {
    let db_manager: ConnectionManager<PgConnection> = ConnectionManager::new(db_url);
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
                list_users,
                settings,
                patch_settings,
                simple_login,
                health_check,
                put_user_team_ranks,
                list_teams,
                join_global_team,
                join_team,
                leave_global_team,
                leave_team,
                delete_team,
                patch_team,
                create_team,
                show_team,
                put_user_capability,
                delete_user_capability,
                google_login,
                login_redirect_google
            ],
        )
        .mount("/public", FileServer::from("./public"))
        .manage(sessions)
        .manage(config)
        .manage(oidc_service)
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
