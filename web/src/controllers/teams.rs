use std::cmp::Ordering;

use diesel::prelude::*;
use rocket::{http::Status, State};
use rocket_dyn_templates::Template;
use serde_json::json;

use crate::{
    guards::SessionId,
    models::{
        features::get_global_features,
        teams::Team,
        users::{should_be_logged_in_if_features, Right, Sessions},
        AppError,
    },
    schema::teams::dsl,
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

    should_be_logged_in_if_features(&Right::Admin, &session_id, sessions, &features, &conn)?;

    let mut teams = dsl::teams.load::<Team>(&conn).map_err(AppError::from)?;

    teams.sort_by(|a, b| {
        if a.slug.is_empty() {
            Ordering::Less
        } else if b.slug.is_empty() {
            Ordering::Greater
        } else {
            a.title.cmp(&b.title)
        }
    });

    Ok(Template::render(
        "teams",
        json!({ "teams": json!(teams).to_string() }),
    ))
}
