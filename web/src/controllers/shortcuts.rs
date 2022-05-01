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
use crate::models::shortcuts::{sorted, NewShortcut, UpdatableShortcut, SHORTCUT_COLUMNS, Shortcut};
use crate::models::teams::{admin_teams}; 
use crate::models::users::{Capability, User};
use crate::models::AppError;
use crate::schema::shortcuts;
use crate::schema::shortcuts::dsl;
use crate::schema::users_teams;
use crate::DbPool;
use crate::views::IndexContext;

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
            admin_teams(user, &conn)?
        )
    } else {
        None
    };

    Ok(Template::render(
        "index",
        json!({ 
            "mail": user.as_ref().map(|user| &user.mail),
            "features": json!(features),
            "context": json!(IndexContext {
                shortcut: None,
                shortcuts:  sorted(&conn)?,
                 user, 
                features,
                team: None,
                teams: admin_teams,
            }).to_string()
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

    let shortcut_found = if let Some(user) = &user && features.teams {
        dsl::shortcuts
            .inner_join(
                users_teams::table.on(dsl::team_slug
                    .eq(users_teams::team_slug)
                    .and(users_teams::user_mail.eq(&user.mail))),
            )
            .filter(dsl::shortcut.eq(shortcut))
            .select(SHORTCUT_COLUMNS)
            .order_by(users_teams::rank.asc())
            .first::<Shortcut>(&conn)
            .optional()
            .map_err(AppError::from)?
    } else {
        dsl::shortcuts
            .find((shortcut, ""))
            .first::<Shortcut>(&conn)
            .optional()
            .map_err(AppError::from)?
    };

    let admin_teams = if let Some(user) = &user && features.teams {
        Some(
            admin_teams(user, &conn)?
            
        )
    } else {
        None
    };

    Ok(match shortcut_found {
        Some(shortcut_found) => {
            if let Some(true) = no_redirect {
                ShortcutRes::Ok(Template::render(
                    "index",
                    json!({ 
                        "mail": user.as_ref().map(|user| &user.mail),
                        "context": json!(IndexContext {
                            shortcut: Some(shortcut_found),
                            shortcuts:  sorted(&conn)?,
                             user, 
                            features,
                            team: None,
                            teams: admin_teams,
                        }).to_string()
                    }),
                ))
            } else {
                ShortcutRes::Redirect(Redirect::permanent(shortcut_found.url))
            }
        }
        None => ShortcutRes::NotFound(Template::render(
            "index",
            json!({ 
                "mail": user.as_ref().map(|user| &user.mail),
                "features": json!(features),
                "not_found": true,
                "context": json!(IndexContext {
                    shortcut: Some(Shortcut {
                        shortcut: shortcut.to_string(),
                        team_slug:"".to_string(), 
                        url:"".to_string()
                    }),
                    shortcuts:  sorted(&conn)?,
                     user, 
                    features,
                    team: None,
                    teams: admin_teams,
                }).to_string()
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
    user.should_have_capability(Capability::ShortcutsWrite)?;

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
    user.should_have_capability(Capability::ShortcutsWrite)?;

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
