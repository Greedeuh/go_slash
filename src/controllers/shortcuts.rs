use log::error;
use rocket::{form::Form, http::Status, response::Redirect, State};
use rocket_dyn_templates::Template;
use serde_json::json;
use std::path::PathBuf;

use crate::Entries;

#[derive(Responder)]
#[allow(clippy::large_enum_variant)]
pub enum ShortcutRes {
    Redirect(Redirect),
    #[response(status = 404)]
    Template(Template),
}

#[get("/<shortcut..>")]
pub fn shortcuts(shortcut: PathBuf, entries: &State<Entries>) -> Result<ShortcutRes, Status> {
    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    Ok(match entries.find(shortcut) {
        Some(url) => ShortcutRes::Redirect(Redirect::permanent(url)),
        None => {
            let template = Template::render(
                "shortcut",
                json!({
                    "shortcut":shortcut,
                    "not_found":true
                }),
            );

            ShortcutRes::Template(template)
        }
    })
}

#[derive(FromForm)]
pub struct Url {
    url: String,
}

#[post("/<shortcut..>", data = "<form>")]
pub fn post_shortcuts(
    shortcut: PathBuf,
    entries: &State<Entries>,
    form: Form<Url>,
) -> Result<(Status, Template), Status> {
    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    let url = form.into_inner().url;
    entries.put(shortcut, url);

    let template = Template::render(
        "shortcut",
        json!({
            "shortcut":shortcut,
            "saved": true,
        }),
    );

    Ok((Status::Created, template))
}

fn parse_shortcut_path_buff(shortcut: &'_ PathBuf) -> Result<&'_ str, Status> {
    match shortcut.to_str() {
        Some(shortcut) => Ok(shortcut),
        None => {
            error!("GET <shortcut..> failed parsing: {:?}", shortcut);
            Err(Status::BadRequest)
        }
    }
}
