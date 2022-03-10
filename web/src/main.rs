#[macro_use]
extern crate rocket;
use std::env;

use go_web::{server, Entries, GlobalFeatures};

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

    let port = env::var("PORT");
    let port = match port {
        Ok(port) => port.parse().unwrap(),
        _ => 8000,
    };

    server(port, entries, features)
}
