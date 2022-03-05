#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_dyn_templates;
use rocket::{
    fs::{relative, FileServer},
    State,
};
use rocket_dyn_templates::Template;
use serde_json::json;

mod controllers;
use controllers::shortcuts::{delete_shortcut, post_shortcuts, shortcuts};
mod models;
pub use models::*;

#[get("/")]
fn index(entries: &State<Entries>) -> Template {
    let mut all_shortcuts = entries
        .all()
        .into_iter()
        .map(|(shortcut, url)| (shortcut, url))
        .collect::<Vec<_>>();

    all_shortcuts.sort_by(|(shortcut_1, _), (shortcut_2, _)| shortcut_1.cmp(shortcut_2));

    let all_shortcuts = all_shortcuts
        .iter()
        .map(|(shortcut, url)| json!({"shortcut": shortcut, "url": url}))
        .collect::<Vec<_>>();

    let all_shortcuts: String = json!(all_shortcuts).to_string();

    Template::render("index", json!({ "shortcuts": all_shortcuts }))
}

pub fn server(entries: Entries) -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount(
            "/",
            routes![index, shortcuts, post_shortcuts, delete_shortcut],
        )
        .mount("/public", FileServer::from(relative!("public")))
        .manage(entries)
        .attach(Template::fairing())
}
