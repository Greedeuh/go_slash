use lazy_static::lazy_static;
use log::error;
use regex::Regex;
use rocket::{form::Form, http::Status, response::Redirect, State};
use rocket_dyn_templates::Template;
use serde_json::json;
use std::path::PathBuf;

use crate::Entries;

lazy_static! {
    static ref URL_REGEX: Regex =
        Regex::new(r#"https?://(www\.)?[-a-zA-Z0-9()@:%_\+.~#?&//=]{1,256}"#,).unwrap();
}

#[derive(Responder)]
#[allow(clippy::large_enum_variant)]
pub enum ShortcutRes {
    Redirect(Redirect),
    #[response(status = 404)]
    NotFound(Template),
    Ok(Template),
}

// rank 11 because static file at /public are at 10 by default
#[get("/<shortcut..>?<no_redirect>", rank = 11)]
pub fn shortcuts(
    shortcut: PathBuf,
    no_redirect: Option<bool>,
    entries: &State<Entries>,
) -> Result<ShortcutRes, Status> {
    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    Ok(match entries.find(shortcut) {
        Some(url) => {
            if let Some(true) = no_redirect {
                ShortcutRes::Ok(Template::render(
                    "shortcut",
                    json!({
                        "shortcut":shortcut,
                        "url": url
                    }),
                ))
            } else {
                ShortcutRes::Redirect(Redirect::permanent(url))
            }
        }
        None => ShortcutRes::NotFound(Template::render(
            "shortcut",
            json!({
                "shortcut":shortcut,
                "not_found":true
            }),
        )),
    })
}

#[derive(FromForm)]
pub struct Url {
    url: String,
}

#[post("/<shortcut..>", data = "<url>")]
pub fn post_shortcuts(
    shortcut: PathBuf,
    entries: &State<Entries>,
    url: Form<Url>,
) -> Result<(Status, Template), Status> {
    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    let url = url.into_inner().url;
    if !URL_REGEX.is_match(&url) {
        return Ok((
            Status::BadRequest,
            Template::render(
                "shortcut",
                json!({
                    "shortcut":shortcut,
                    "wrong_url": true,
                }),
            ),
        ));
    }

    entries.put(shortcut, url);

    Ok((
        Status::Created,
        Template::render(
            "shortcut",
            json!({
                "shortcut":shortcut,
                "saved": true,
            }),
        ),
    ))
}

#[delete("/<shortcut..>")]
pub fn delete_shortcut(shortcut: PathBuf, entries: &State<Entries>) -> Result<Template, Status> {
    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    entries.delete(shortcut);

    Ok(Template::render(
        "shortcut",
        json!({
            "shortcut":shortcut,
            "deleted":true
        }),
    ))
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
