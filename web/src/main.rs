#[macro_use]
extern crate rocket;
use std::env;

use go_web::{server, AppConfig, Entries, GlobalFeatures, SimpleUsers};

#[launch]
fn rocket() -> _ {
    let file = env::var("SHORTCUTS_FILE");
    let entries = match file {
        Ok(file) => Entries::from_path(&file),
        _ => Entries::from_path("shortcuts.yaml"),
    };

    let file = env::var("FEATURE_FILE");
    let features = match file {
        Ok(file) => GlobalFeatures::from_path(&file),
        _ => GlobalFeatures::from_path("features.yaml"),
    };

    let file = env::var("SIMPLE_USERS_FILE");
    let users = match file {
        Ok(file) => SimpleUsers::from_path(&file),
        _ => SimpleUsers::from_path("simple_users.yaml"),
    };

    let port = env::var("PORT");
    let port = match port {
        Ok(port) => port.parse().unwrap(),
        _ => 8000,
    };

    let simple_login_salt1 = env::var("SALT1").expect("expect env var SALT1");
    let simple_login_salt2 = env::var("SALT2").expect("expect env var SALT2");

    server(
        port,
        entries,
        features,
        users,
        AppConfig {
            simple_login_salt1,
            simple_login_salt2,
        },
    )
}
