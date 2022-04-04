use diesel::prelude::*;
use rocket::{http::Status, State};
use rocket_dyn_templates::Template;
use serde_json::json;
use std::cmp::Ordering;

use crate::{
    guards::SessionId,
    models::{
        features::get_global_features,
        teams::TeamForOptUser,
        users::{should_be_logged_in_with, Right, Sessions},
        AppError,
    },
    schema::{teams::dsl, users_teams},
    DbPool,
};

#[get("/go/teams")]
pub fn list_teams(
    session_id: Option<SessionId>,
    sessions: &State<Sessions>,
    pool: &State<DbPool>,
) -> Result<Template, (Status, Template)> {
    let conn = pool.get().map_err(AppError::from)?;
    let features = get_global_features(&conn)?;

    if !features.teams {
        return Err(AppError::Disable.into());
    }

    let user = should_be_logged_in_with(&Right::Read, &session_id, sessions, &features, &conn)?;

    let mut teams: Vec<TeamForOptUser> = dsl::teams
        .left_join(
            users_teams::table.on(dsl::slug
                .eq(users_teams::team_slug)
                .and(users_teams::user_mail.eq(&user.mail))),
        )
        .load(&conn)
        .map_err(AppError::from)?;

    teams.sort_by(
        |TeamForOptUser { team: a, .. }, TeamForOptUser { team: b, .. }| {
            if a.slug.is_empty() {
                Ordering::Less
            } else if b.slug.is_empty() {
                Ordering::Greater
            } else {
                a.title.cmp(&b.title)
            }
        },
    );

    Ok(Template::render(
        "teams",
        json!({ "teams": json!(teams).to_string(), "mail": &user.mail }),
    ))
}
