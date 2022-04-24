use diesel::{dsl::max, prelude::*};
use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde::Deserialize;
use serde_json::json;
use std::cmp::Ordering;

use crate::{
    models::{
        features::Features,
        teams::{Team, TeamForOptUser},
        users::{should_have_capability, Capability, User, UserTeam},
        AppError,
    },
    schema::{
        teams::{self, dsl},
        users_teams,
    },
    DbPool,
};

#[get("/go/teams")]
pub fn list_teams(
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Template, (Status, Template)> {
    if !features.teams || !features.login.simple {
        return Err(AppError::Disable.into());
    }

    should_have_capability(&user, Capability::TeamsRead)?;

    let conn = pool.get().map_err(AppError::from)?;
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
        json!({ "teams": json!(teams).to_string(),
            "features": json!(features),
            "capabilities": json!(user.capabilities).to_string(),
            "mail": user.mail
        }),
    ))
}

#[delete("/go/teams/<team>")]
pub fn delete_team(
    team: String,
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    if !features.teams || !features.login.simple {
        return Err(AppError::Disable.into());
    }

    should_have_capability(&user, Capability::TeamsWrite)?;

    let conn = pool.get().map_err(AppError::from)?;
    diesel::delete(teams::table.find(team))
        .execute(&conn)
        .map_err(AppError::from)?;

    Ok(Status::Ok)
}

#[derive(Deserialize)]
pub struct NewTeam {
    pub slug: String,
    pub title: String,
    pub is_private: bool,
}

#[post("/go/teams", data = "<data>")]
pub fn create_team(
    data: Json<NewTeam>,
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    if !features.teams || !features.login.simple {
        return Err(AppError::Disable.into());
    }

    let conn = pool.get().map_err(AppError::from)?;
    conn.transaction::<_, diesel::result::Error, _>(|| {
        let NewTeam {
            slug,
            title,
            is_private,
        } = data.into_inner();
        let team = Team {
            slug: slug.clone(),
            is_accepted: user.capabilities.contains(&Capability::TeamsWrite),
            title,
            is_private,
        };
        diesel::insert_into(teams::table)
            .values(team)
            .execute(&conn)?;

        let previous_rank = (users_teams::table
            .select(max(users_teams::rank))
            .filter(users_teams::user_mail.eq(&user.mail))
            .first::<Option<i16>>(&conn)?)
        .unwrap_or(0);

        let user_team = UserTeam {
            user_mail: user.mail,
            team_slug: slug,
            is_admin: true,
            is_accepted: true,
            rank: previous_rank as i16 + 1,
        };
        diesel::insert_into(users_teams::table)
            .values(user_team)
            .execute(&conn)?;
        Ok(())
    })
    .map_err(AppError::from)?;

    Ok(Status::Created)
}

#[derive(Deserialize, AsChangeset)]
#[table_name = "teams"]
pub struct PatchableTeam {
    pub title: Option<String>,
    pub is_private: Option<bool>,
    pub is_accepted: Option<bool>,
}

#[patch("/go/teams/<team>", data = "<data>")]
pub fn patch_team(
    team: String,
    data: Json<PatchableTeam>,
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    if !features.teams || !features.login.simple {
        return Err(AppError::Disable.into());
    }

    should_have_capability(&user, Capability::TeamsWrite)?;

    let conn = pool.get().map_err(AppError::from)?;
    diesel::update(teams::table.find(team))
        .set(&data.into_inner())
        .execute(&conn)
        .map_err(AppError::from)?;

    Ok(Status::Ok)
}
