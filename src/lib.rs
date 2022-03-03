#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_dyn_templates;
use rocket_dyn_templates::Template;

mod controllers;
use controllers::shortcuts::{delete_shortcut, post_shortcuts, shortcuts};
mod models;
pub use models::*;

#[get("/")]
fn index() -> &'static str {
    ""
}

pub fn server(entries: Entries) -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount(
            "/",
            routes![index, shortcuts, post_shortcuts, delete_shortcut],
        )
        .manage(entries)
        .attach(Template::fairing())
}
