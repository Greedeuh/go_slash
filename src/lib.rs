#[macro_use]
extern crate rocket;

mod controllers;
use controllers::shortcuts::shortcuts;
mod models;
pub use models::*;

#[get("/")]
fn index() -> &'static str {
    ""
}

pub fn server(entries: Entries) -> rocket::Rocket<rocket::Build> {
    env_logger::init();

    rocket::build()
        .mount("/", routes![index, shortcuts])
        .manage(entries)
}
