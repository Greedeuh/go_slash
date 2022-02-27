#[macro_use]
extern crate rocket;
use std::collections::HashMap;

use go::{server, Entries};

#[launch]
fn run() -> _ {
    server(Entries::new(HashMap::new()))
}
