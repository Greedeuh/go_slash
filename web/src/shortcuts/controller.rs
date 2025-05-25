use diesel::dsl::count;
use diesel::prelude::*;
use lazy_static::lazy_static;
use log::error;
use regex::Regex;
use rocket::serde::{json::Json, Deserialize};
use rocket::{http::Status, response::Redirect, State};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

use crate::shortcuts::{
    NewShortcut, Shortcut, UpdatableShortcut, SHORTCUT_COLUMNS,
};
use crate::teams::{
     user_should_have_team_capability, Team, TeamCapability
};
use crate::users::User;
use crate::errors::AppError;
use crate::schema::shortcuts::dsl;
use crate::schema::users_teams;
use crate::schema::{shortcuts, teams};
use crate::views::IndexContext;
use crate::{DbConn, DbPool};

lazy_static! {
    static ref URL_REGEX: Regex =
        Regex::new(r#"https?://(www\.)?[-a-zA-Z0-9()@:%_\+.~#?&//=]{1,256}"#,).unwrap();
}

#[get("/")]
pub fn index(user: User, pool: &State<DbPool>) -> Result<Template, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    let teams = Team::all_with_shortcut_write(&user, &mut conn)?;
    let shortcuts = Shortcut::sorted(&user, &mut conn)?;

    Ok(Template::render(
        "index",
        json!({
            "mail": &user.mail,
            "context": json!(IndexContext {
                shortcut: None,
                shortcuts: shortcuts,
                 user,
                team: None,
                teams: teams,
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
    user: User,

    pool: &State<DbPool>,
) -> Result<ShortcutRes, (Status, Template)> {
    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    let mut conn = pool.get().map_err(AppError::from)?;

    let shortcut_found = dsl::shortcuts
        .inner_join(
            users_teams::table.on(dsl::team_slug
                .eq(users_teams::team_slug)
                .and(users_teams::user_mail.eq(&user.mail))
                .and(users_teams::is_accepted)),
        )
        .filter(dsl::shortcut.eq(shortcut))
        .select(SHORTCUT_COLUMNS)
        .order_by(users_teams::rank.asc())
        .first::<Shortcut>(&mut conn)
        .optional()
        .map_err(AppError::from)?;

    let admin_teams = Team::all_with_shortcut_write(&user, &mut conn)?;

    Ok(match shortcut_found {
        Some(shortcut_found) => {
            if let Some(true) = no_redirect {
                ShortcutRes::Ok(Template::render(
                    "index",
                    json!({
                        "mail": &user.mail,
                        "context": json!(IndexContext {
                            shortcut: Some(shortcut_found),
                            shortcuts:  Shortcut::sorted(&user, &mut conn)?,
                            user,
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
                "mail":&user.mail,
                "not_found": true,
                "context": json!(IndexContext {
                    shortcut: Some(Shortcut {
                        shortcut: shortcut.to_string(),
                        team_slug:"".to_string(),
                        url:"".to_string()
                    }),
                    shortcuts: Shortcut::sorted(&user, &mut conn)?,
                    user,
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

    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    let data = data.into_inner();
    let url = data.url;
    if !URL_REGEX.is_match(&url) {
        return Err((Status::BadRequest, json!({"error": "Wrong URL format."})));
    }

    let team_slug = if let Some(team) = team {
        team
    } else {
        "".to_string()
    };

    let mut conn = pool.get().map_err(AppError::from)?;

    team_should_be_accepted(&team_slug, &mut conn)?;
    user_should_have_team_capability(&user, &team_slug, &mut conn, TeamCapability::ShortcutsWrite)?;

    diesel::insert_into(shortcuts::table)
        .values(NewShortcut {
            shortcut: shortcut.to_string(),
            url: url.to_string(),
            team_slug: team_slug.to_string(),
        })
        .on_conflict((shortcuts::shortcut, shortcuts::team_slug))
        .do_update()
        .set(UpdatableShortcut { url, team_slug })
        .execute(&mut conn)
        .map_err(AppError::from)?;

    Ok(Status::Ok)
}

#[delete("/<shortcut..>?<team>")]
pub fn delete_shortcut(
    shortcut: PathBuf,
    team: Option<String>,
    user: User,

    pool: &State<DbPool>,
) -> Result<Template, (Status, Value)> {
    let shortcut = parse_shortcut_path_buff(&shortcut)?;

    let team_slug = if let Some(team) = team {
        team
    } else {
        "".to_string()
    };

    let mut conn = pool.get().map_err(AppError::from)?;

    team_should_be_accepted(&team_slug, &mut conn)?;
    user_should_have_team_capability(&user, &team_slug, &mut conn, TeamCapability::ShortcutsWrite)?;

    diesel::delete(shortcuts::table)
        .filter(dsl::shortcut.eq(shortcut).and(dsl::team_slug.eq(team_slug)))
        .execute(&mut conn)
        .map_err(AppError::from)?;

    Ok(Template::render(
        "index",
        json!({
            "shortcut":shortcut,
            "deleted":true,
        }),
    ))
}

fn parse_shortcut_path_buff(shortcut: &'_ Path) -> Result<&'_ str, AppError> {
    match shortcut.to_str() {
        Some(shortcut) => Ok(shortcut),
        None => {
            error!("GET <shortcut..> failed parsing: {shortcut:?}");
            Err(AppError::BadRequest)
        }
    }
}

fn team_should_be_accepted(team_slug: &str, conn: &mut DbConn) -> Result<(), AppError> {
    if teams::table
        .find(&team_slug)
        .filter(teams::is_accepted)
        .select(count(teams::slug))
        .first::<i64>(conn)
        .map_err(AppError::from)?
        == 0
    {
        return Err(AppError::Unauthorized);
    }
    Ok(())
}
