use diesel::prelude::*;
use lazy_static::lazy_static;
use log::error;
use regex::Regex;
use rocket::serde::{json::Json, Deserialize};
use rocket::{http::Status, response::Redirect, State};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

use crate::guards::SessionId;
use crate::models::features::get_global_features;
use crate::models::shortcuts::{sorted, NewShortcut};
use crate::models::users::{read_or_write, should_be_logged_in_if_features, Right, Sessions};
use crate::models::AppError;
use crate::schema::shortcuts;
use crate::schema::shortcuts::dsl;
use crate::DbPool;

lazy_static! {
    static ref URL_REGEX: Regex =
        Regex::new(r#"https?://(www\.)?[-a-zA-Z0-9()@:%_\+.~#?&//=]{1,256}"#,).unwrap();
}

#[get("/")]
pub fn index(
    session_id: Option<SessionId>,
    sessions: &State<Sessions>,
    pool: &State<DbPool>,
) -> Result<Template, (Status, Template)> {
    let conn = pool.get().map_err(AppError::from)?;
    let features = get_global_features(&conn)?;

    let user =
        should_be_logged_in_if_features(&Right::Read, &session_id, sessions, &features, &conn)?;
    let user_mail = user.map(|u| u.mail);

    let right = read_or_write(&features, &user_mail)?;

    Ok(Template::render(
        "index",
        json!({ "shortcuts": json!(sorted(&conn)?).to_string(), "right": right, "mail": user_mail }),
    ))
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
pub fn get_shortcut(
    shortcut: PathBuf,
    no_redirect: Option<bool>,
    session_id: Option<SessionId>,
    sessions: &State<Sessions>,
    pool: &State<DbPool>,
) -> Result<ShortcutRes, (Status, Template)> {
    let conn = pool.get().map_err(AppError::from)?;
    let features = get_global_features(&conn)?;

    let user =
        should_be_logged_in_if_features(&Right::Read, &session_id, sessions, &features, &conn)?;
    let user_mail = user.map(|u| u.mail);
    let right = read_or_write(&features, &user_mail)?;

    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    let conn = pool.get().map_err(AppError::from)?;

    let url = dsl::shortcuts
        .select(dsl::url)
        .find(shortcut)
        .first::<String>(&conn)
        .optional()
        .map_err(AppError::from)?;

    Ok(match url {
        Some(url) => {
            if let Some(true) = no_redirect {
                ShortcutRes::Ok(Template::render(
                    "index",
                    json!({
                        "shortcut": shortcut,
                        "url": url,
                        "shortcuts": json!(sorted(&conn)?)
                            .to_string(),
                        "url": url,
                        "no_redirect": true,
                        "right": right,
                        "mail": user_mail
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
                "shortcuts": json!(sorted(&conn)?)
                                    .to_string(),
                "not_found": true,
                "right": right,
                "mail": user_mail
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
    url: Json<Url>,
    session_id: Option<SessionId>,
    sessions: &State<Sessions>,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    let conn = pool.get().map_err(AppError::from)?;
    let features = get_global_features(&conn)?;

    should_be_logged_in_if_features(&Right::Write, &session_id, sessions, &features, &conn)?;

    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    let url = url.into_inner().url;
    if !URL_REGEX.is_match(&url) {
        return Err((Status::BadRequest, json!({"error": "Wrong URL format."})));
    }

    let conn = pool.get().map_err(AppError::from)?;

    diesel::replace_into(shortcuts::table)
        .values(NewShortcut {
            shortcut: shortcut.to_string(),
            url,
            team_slug: "".to_string(),
        })
        .execute(&conn)
        .map_err(AppError::from)?;

    Ok(Status::Ok)
}

#[delete("/<shortcut..>")]
pub fn delete_shortcut(
    shortcut: PathBuf,
    session_id: Option<SessionId>,
    sessions: &State<Sessions>,
    pool: &State<DbPool>,
) -> Result<Template, (Status, Value)> {
    let conn = pool.get().map_err(AppError::from)?;
    let features = get_global_features(&conn)?;

    should_be_logged_in_if_features(&Right::Write, &session_id, sessions, &features, &conn)?;

    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    let conn = pool.get().map_err(AppError::from)?;

    diesel::delete(shortcuts::table)
        .filter(dsl::shortcut.eq(shortcut))
        .execute(&conn)
        .map_err(AppError::from)?;

    Ok(Template::render(
        "index",
        json!({
            "shortcut":shortcut,
            "deleted":true
        }),
    ))
}

fn parse_shortcut_path_buff(shortcut: &'_ Path) -> Result<&'_ str, AppError> {
    match shortcut.to_str() {
        Some(shortcut) => Ok(shortcut),
        None => {
            error!("GET <shortcut..> failed parsing: {:?}", shortcut);
            Err(AppError::BadRequest)
        }
    }
}
