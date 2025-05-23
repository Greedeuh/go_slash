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
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use rocket::{fairing::AdHoc, fs::FileServer, http::Status, routes, Build, Config, Rocket};
use rocket_dyn_templates::Template;

pub mod guards;
pub mod errors;
pub mod users;
pub mod teams;
pub mod shortcuts;
pub mod schema;
pub mod services;
mod views;
pub mod settings;
pub mod login;

use teams::{
    create_team, delete_team, delete_user_link_capability, kick_user, list_teams, patch_team,
    put_user_link_capability, put_user_team_acceptation, show_team,
};
use settings::{patch_settings, get_settings};
use login::{google_login, login as go_login, login_redirect_google, simple_login};
use shortcuts::{delete_shortcut, get_shortcut, index, put_shortcut};
    use users::{
        delete_user_capability, join_global_team, join_team, leave_global_team, leave_team,
        list_users, put_user_capability, put_user_team_ranks,
    };


use crate::{
    guards::UnauthorizedAsLogin, users::Sessions, services::oidc::OidcService,
};


pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

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
        let mut conn = db_pool.get().unwrap();
        conn.run_pending_migrations(MIGRATIONS).unwrap();
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
                go_login,
                list_users,
                get_settings,
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
                login_redirect_google,
                kick_user,
                put_user_link_capability,
                delete_user_link_capability,
                put_user_team_acceptation
            ],
        )
        .mount("/public", FileServer::from("./public"))
        .manage(sessions)
        .manage(config)
        .manage(oidc_service)
        .manage(db_pool)
        .attach(Template::fairing())
        .attach(UnauthorizedAsLogin {})
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

#[get("/go/health")]
pub fn health_check() -> Status {
    Status::Ok
}