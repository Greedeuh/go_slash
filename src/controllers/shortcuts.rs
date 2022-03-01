use log::error;
use rocket_dyn_templates::Template;
use std::{collections::HashMap, path::PathBuf};

use rocket::{http::Status, response::Redirect, State};

use crate::Entries;

#[derive(Responder)]
#[allow(clippy::large_enum_variant)]
pub enum ShortcutRes {
    Redirect(Redirect),
    Status(Status),
    #[response(status = 404)]
    Template(Template),
}

#[get("/<shortcut..>")]
pub fn shortcuts(shortcut: PathBuf, entries: &State<Entries>) -> ShortcutRes {
    let shortcut = match shortcut.to_str() {
        Some(shortcut) => shortcut,
        None => {
            error!("GET <shortcut..> failed parsing: {:?}", shortcut);
            return ShortcutRes::Status(Status::BadRequest);
        }
    };

    match entries.find(shortcut) {
        Some(url) => ShortcutRes::Redirect(Redirect::permanent(url)),
        None => {
            let mut context = HashMap::new();
            context.insert("shortcut", &shortcut);
            let template = Template::render("shortcut_not_found", context);

            ShortcutRes::Template(template)
        }
    }
}
