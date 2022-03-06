#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_dyn_templates;
use rocket::{
    fs::{relative, FileServer},
    http::Status,
    Build, Rocket, State,
};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};

mod controllers;
use controllers::shortcuts::{delete_shortcut, put_shortcut, shortcuts};
mod models;
pub use models::*;

#[get("/")]
fn index(entries: &State<Entries>) -> Result<Template, (Status, Value)> {
    let all_shortcuts = entries.sorted()?;

    let all_shortcuts = all_shortcuts
        .iter()
        .map(|(shortcut, url)| json!({"shortcut": shortcut, "url": url}))
        .collect::<Vec<_>>();

    let all_shortcuts: String = json!(all_shortcuts).to_string();

    Ok(Template::render(
        "index",
        json!({ "shortcuts": all_shortcuts }),
    ))
}

pub fn server(entries: Entries) -> Rocket<Build> {
    rocket::build()
        .mount(
            "/",
            routes![index, shortcuts, put_shortcut, delete_shortcut],
        )
        .mount("/public", FileServer::from(relative!("public")))
        .manage(entries)
        .attach(Template::fairing())
}
