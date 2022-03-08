#[macro_use]
extern crate rocket;
use std::env;

use go::{server, Entries};

#[launch]
fn rocket() -> _ {
    let file = env::var("SHORTCUTS_FILE");
    let entries = match file {
        Ok(file) => Entries::from_path(&file),
        _ => Entries::from_path("shortcuts.yaml"),
    };

    server(entries)
}
