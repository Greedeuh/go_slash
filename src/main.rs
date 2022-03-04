#[macro_use]
extern crate rocket;
use std::{collections::HashMap, env, fs};

use go::{server, Entries};

#[launch]
fn rocket() -> _ {
    let file = env::var("SHORTCUTS_FILE");
    let entries = match file {
        Ok(file) => Entries::from(fs::read_to_string(file).unwrap().as_str()),
        _ => Entries::new(HashMap::new()),
    };

    server(entries)
}
