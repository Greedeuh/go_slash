use diesel::prelude::*;
use lazy_static::lazy_static;
use log::error;
use regex::Regex;
use rocket::serde::{json::Json, Deserialize};
use rocket::{http::Status, response::Redirect, State};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

use crate::models::features::Features;
use crate::models::shortcuts::{sorted, NewShortcut, UpdatableShortcut};
use crate::models::teams::{Team, TEAM_COLUMNS};
use crate::models::users::{should_have_capability, Capability, User};
use crate::models::AppError;
use crate::schema::shortcuts;
use crate::schema::shortcuts::dsl;
use crate::schema::teams;
use crate::schema::users_teams;
use crate::DbPool;

lazy_static! {
    static ref URL_REGEX: Regex =
        Regex::new(r#"https?://(www\.)?[-a-zA-Z0-9()@:%_\+.~#?&//=]{1,256}"#,).unwrap();
}

#[get("/")]
pub fn index(
    user: Option<User>,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Template, (Status, Template)> {
    if user.is_none() && features.login.read_private {
        return Err(AppError::Unauthorized.into());
    }

    let conn = pool.get().map_err(AppError::from)?;

    let admin_teams = if let Some(user) = &user && features.teams {
        Some(
            json!(teams::table
                .inner_join(
                    users_teams::table.on(teams::slug
                        .eq(users_teams::team_slug)
                        .and(users_teams::user_mail.eq(&user.mail))
                        .and(users_teams::is_admin.eq(true))),
                )
                .select(TEAM_COLUMNS)
                .load::<Team>(&conn)
                .map_err(AppError::from)?)
            .to_string(),
        )
    } else {
        None
    };

    Ok(Template::render(
        "index",
        json!({ "shortcuts": json!(sorted(&conn)?).to_string(),
            "capabilities": json!(user.as_ref().map(|user| &user.capabilities)).to_string(),
            "mail": user.map(|user| user.mail),
            "features": json!(features),
            "admin_teams": admin_teams
        }),
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
    user: Option<User>,
    features: Features,
    pool: &State<DbPool>,
) -> Result<ShortcutRes, (Status, Template)> {
    if user.is_none() && features.login.read_private {
        return Err(AppError::Unauthorized.into());
    }

    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    let conn = pool.get().map_err(AppError::from)?;

    let url = if let Some(user) = &user && features.teams {
        dsl::shortcuts
            .inner_join(
                users_teams::table.on(dsl::team_slug
                    .eq(users_teams::team_slug)
                    .and(users_teams::user_mail.eq(&user.mail))),
            )
            .filter(dsl::shortcut.eq(shortcut))
            .select(dsl::url)
            .order_by(users_teams::rank.asc())
            .first::<String>(&conn)
            .optional()
            .map_err(AppError::from)?
    } else {
        dsl::shortcuts
            .select(dsl::url)
            .find((shortcut, ""))
            .first::<String>(&conn)
            .optional()
            .map_err(AppError::from)?
    };

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
                        "capabilities": json!(user.as_ref().map(|user| &user.capabilities)).to_string(),
                        "mail": user.map(|user| user.mail),
                        "features": json!(features)
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
                "capabilities": json!(user.as_ref().map(|user| &user.capabilities)).to_string(),
                "mail": user.map(|user| user.mail),
                "features": json!(features)
            }),
        )),
    })
}

#[derive(Deserialize)]
pub struct Url {
    url: String,
}

#[put("/<shortcut..>?<team>", data = "<data>")]
pub fn put_shortcut(
    shortcut: PathBuf,
    team: Option<String>,
    data: Json<Url>,
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    should_have_capability(&user, Capability::ShortcutsWrite)?;

    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    let data = data.into_inner();
    let url = data.url;
    if !URL_REGEX.is_match(&url) {
        return Err((Status::BadRequest, json!({"error": "Wrong URL format."})));
    }

    let team_slug = if features.teams && let Some(team) = team {
        team
    } else {
        "".to_string()
    };

    let conn = pool.get().map_err(AppError::from)?;

    diesel::insert_into(shortcuts::table)
        .values(NewShortcut {
            shortcut: shortcut.to_string(),
            url: url.to_string(),
            team_slug: team_slug.to_string(),
        })
        // .on_conflict(SHORTCUT_COLUMNS)
        // .do_nothing()
        .on_conflict((shortcuts::shortcut, shortcuts::team_slug))
        .do_update()
        .set(UpdatableShortcut { url, team_slug })
        .execute(&conn)
        .map_err(AppError::from)?;

    Ok(Status::Ok)
}

#[delete("/<shortcut..>?<team>")]
pub fn delete_shortcut(
    shortcut: PathBuf,
    team: Option<String>,
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Template, (Status, Value)> {
    should_have_capability(&user, Capability::ShortcutsWrite)?;

    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    let team_slug = if features.teams && let Some(team) = team {
        team
    } else {
        "".to_string()
    };

    let conn = pool.get().map_err(AppError::from)?;

    diesel::delete(shortcuts::table)
        .filter(dsl::shortcut.eq(shortcut).and(dsl::team_slug.eq(team_slug)))
        .execute(&conn)
        .map_err(AppError::from)?;

    Ok(Template::render(
        "index",
        json!({
            "shortcut":shortcut,
            "deleted":true,
            "features": json!(features)
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
