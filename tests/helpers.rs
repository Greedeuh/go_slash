use go::{server, Entries};
use rocket::local::blocking::Client;
use std::collections::HashMap;

pub fn launch_empty() -> Client {
    Client::tracked(server(Entries::new(HashMap::new()))).expect("valid rocket instance")
}

#[allow(dead_code)]
pub fn launch_with(shortcuts: &str) -> Client {
    let shortcuts = shortcuts
        .lines()
        .map(|line| {
            let line = line.replace(' ', "");
            let (key, value) = line
                .split_once(':')
                .expect("launch_with shortcuts failed parsing");
            (key.to_owned(), value.to_owned())
        })
        .collect();

    Client::tracked(server(Entries::new(shortcuts))).expect("valid rocket instance")
}
