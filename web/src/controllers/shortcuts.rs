use lazy_static::lazy_static;
use log::error;
use regex::Regex;
use rocket::serde::{json::Json, Deserialize};
use rocket::{http::Status, response::Redirect, State};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

use crate::{AppError, Entries};

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
) -> Result<ShortcutRes, (Status, Value)> {
    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    Ok(match entries.find(shortcut)? {
        Some(url) => {
            if let Some(true) = no_redirect {
                ShortcutRes::Ok(Template::render(
                    "index",
                    json!({
                        "shortcut": shortcut,
                        "url": url,
                        "shortcuts": json!(entries.sorted()?
                            .iter()
                            .map(|(shortcut, url)| json!({"shortcut": shortcut, "url": url}))
                            .collect::<Vec<_>>())
                            .to_string(),
                        "url": url,
                        "no_redirect": true
                    }),
                ))
            } else {
                ShortcutRes::Redirect(Redirect::permanent(url))
            }
        }
        None => ShortcutRes::NotFound(Template::render(
            "index",
            json!({
                "shortcut": shortcut,
                "shortcuts": json!(entries.sorted()?
                                    .iter()
                                    .map(|(shortcut, url)| json!({"shortcut": shortcut, "url": url}))
                                    .collect::<Vec<_>>())
                                    .to_string(),
                "not_found": true
            }),
        )),
    })
}

#[derive(Deserialize)]
pub struct Url {
    url: String,
}

#[put("/<shortcut..>", data = "<url>")]
pub fn put_shortcut(
    shortcut: PathBuf,
    entries: &State<Entries>,
    url: Json<Url>,
) -> Result<Status, (Status, Value)> {
    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    let url = url.into_inner().url;
    if !URL_REGEX.is_match(&url) {
        return Err((Status::BadRequest, json!({"error": "Wrong URL format."})));
    }

    if entries.put(shortcut, url).is_ok() {};

    Ok(Status::Ok)
}

#[delete("/<shortcut..>")]
pub fn delete_shortcut(
    shortcut: PathBuf,
    entries: &State<Entries>,
) -> Result<Template, (Status, Value)> {
    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    if entries.delete(shortcut).is_ok() {};

    Ok(Template::render(
        "index",
        json!({
            "shortcut":shortcut,
            "deleted":true
        }),
    ))
}

fn parse_shortcut_path_buff(shortcut: &'_ Path) -> Result<&'_ str, (Status, Value)> {
    match shortcut.to_str() {
        Some(shortcut) => Ok(shortcut),
        None => {
            error!("GET <shortcut..> failed parsing: {:?}", shortcut);
            Err((
                Status::BadRequest,
                json!({"error": "Wrong shortcut format."}),
            ))
        }
    }
}

impl From<AppError> for (Status, Value) {
    fn from(e: AppError) -> Self {
        match e {
            AppError::Db => (
                Status::InternalServerError,
                json!({"error": "Probably a database issue :/"}),
            ),
        }
    }
}