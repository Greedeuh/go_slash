use diesel::{
    dsl::{count},
    prelude::*,
};
use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{cmp::Ordering, str::FromStr};

use crate::{
    errors::{
        AppError,
    },
    shortcuts::Shortcut,
    teams::{
         user_should_have_team_capability, Team, TeamCapability,
        TeamForOptUser, TeamForUserIfSome, TeamWithUsers, PatchableTeam
    },
    users::{Capability, User, UserTeam},
    schema::{
        shortcuts,
        teams,
        users_teams,
    },
    views::IndexContext,
    DbPool,
};

#[get("/go/teams")]
pub fn list_teams(user: User, pool: &State<DbPool>) -> Result<Template, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    let mut teams =  Team::all_of_user(&user.mail, &mut conn)?;

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
            "capabilities": json!(user.capabilities).to_string(),
            "mail": user.mail
        }),
    ))
}

#[delete("/go/teams/<slug>")]
pub fn delete_team(
    slug: String,
    user: User,

    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    if !user.have_capability(Capability::TeamsWrite) {
        user_should_have_team_capability(&user, &slug, &mut conn, TeamCapability::TeamsWrite)?;
    }

    Team::delete(&slug, &mut conn)?;

    Ok(Status::Ok)
}

#[derive(Deserialize)]
pub struct NewTeam {
    pub slug: String,
    pub title: String,
    pub is_private: bool,
}

#[post("/go/teams", data = "<new_team>")]
pub fn create_team(
    new_team: Json<NewTeam>,
    user: User,

    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    user.should_have_one_of_theses_capabilities(&[
        Capability::TeamsWrite,
        Capability::TeamsWriteWithValidation,
    ])?;

    let mut conn = pool.get().map_err(AppError::from)?;
    Team::create(new_team.into_inner(), &user, &mut conn)?;

    Ok(Status::Created)
}

#[patch("/go/teams/<team>", data = "<data>")]
pub fn patch_team(
    team: String,
    data: Json<PatchableTeam>,
    user: User,

    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    // should be admin or (part of the team but can't change is_accepted)
    let global_right = user.should_have_capability(Capability::TeamsWrite);
    if let Err(err) = global_right
        && (data.is_accepted.is_some() || users_teams::table
            .find((&user.mail, &team))
            .filter(users_teams::capabilities.contains(vec![TeamCapability::TeamsWrite]))
            .select(count(users_teams::user_mail))
            .first::<i64>(&mut conn)
            .map_err(AppError::from)?
            != 1)
    {
        return Err(err.into());
    }

    Team::update(data.into_inner(), &team, &mut conn)?;

    Ok(Status::Ok)
}

#[get("/go/teams/<slug>")]
pub fn show_team(
    slug: String,
    user: User,
    pool: &State<DbPool>,
) -> Result<Template, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    let team_with_user_if_some: Option<TeamForUserIfSome> = Team::find(&slug, &mut conn)?;

    if team_with_user_if_some.is_none()
        || (!user.have_capability(Capability::TeamsWrite)
            && team_with_user_if_some.as_ref().unwrap().team.is_private
            && team_with_user_if_some.as_ref().unwrap().user_link.is_none())
    {
        return Err(AppError::NotFound.into());
    }

    let shortcuts = shortcuts::table
        .filter(shortcuts::team_slug.eq(&slug))
        .order(shortcuts::shortcut.asc())
        .load::<Shortcut>(&mut conn)
        .map_err(AppError::from)?;

    let team: Team = teams::table
        .find(&slug)
        .first(&mut conn)
        .map_err(AppError::from)?;

    let user_links: Vec<UserTeam> = users_teams::table
        .filter(users_teams::team_slug.eq(slug))
        .order(users_teams::user_mail)
        .load(&mut conn)
        .map_err(AppError::from)?;

    let team = TeamWithUsers { team, user_links };

    Ok(Template::render(
        "index",
        json!({
            "mail": &user.mail,
            "context": json!(IndexContext {
                shortcut: None,
                shortcuts,
                team: Some(team),
                teams: Team::all_with_shortcut_write(&user, &mut conn)?,
                user
            }).to_string()
        }),
    ))
}

#[delete("/go/teams/<slug>/users/<mail>")]
pub fn kick_user(
    slug: String,
    mail: String,
    user: User,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    // should be admin or (part of the team but can't change is_accepted)
    if !user.have_capability(Capability::TeamsWrite) {
        user_should_have_team_capability(&user, &slug, &mut conn, TeamCapability::TeamsWrite)?;
    }

    Team::kick_user(&slug, &mail, &mut conn)?;

    Ok(Status::Ok)
}

#[put("/go/teams/<team_slug>/users/<mail>/capabilities/<capability>")]
pub fn put_user_link_capability(
    team_slug: String,
    mail: String,
    capability: String,
    user: User,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    let capability = TeamCapability::from_str(&capability).map_err(|_| AppError::BadRequest)?;

    let mut conn = pool.get().map_err(AppError::from)?;

    if !user.have_capability(Capability::TeamsWrite) {
        user_should_have_team_capability(&user, &team_slug, &mut conn, TeamCapability::TeamsWrite)?;
    }

    Team::add_user_capability(&mail, &team_slug, capability, &mut conn)?;

    Ok(Status::Ok)
}

#[delete("/go/teams/<team_slug>/users/<mail>/capabilities/<capability>")]
pub fn delete_user_link_capability(
    team_slug: String,
    mail: String,
    capability: String,
    user: User,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    let capability = TeamCapability::from_str(&capability).map_err(|_| AppError::BadRequest)?;

    let mut conn = pool.get().map_err(AppError::from)?;

    if !user.have_capability(Capability::TeamsWrite) {
        user_should_have_team_capability(&user, &team_slug, &mut conn, TeamCapability::TeamsWrite)?;
    }

    Team::remove_user_capability(&mail, &team_slug, capability, &mut conn)?;

    Ok(Status::Ok)
}

#[put("/go/teams/<team_slug>/users/<mail>/is_accepted/<acceptation>")]
pub fn put_user_team_acceptation(
    team_slug: String,
    mail: String,
    acceptation: bool,
    user: User,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    if !user.have_capability(Capability::TeamsWrite) {
        user_should_have_team_capability(&user, &team_slug, &mut conn, TeamCapability::TeamsWrite)?;
    }

    Team::set_acceptation_user(&mail, &team_slug,&acceptation, &mut conn)?;

    Ok(Status::Ok)
}
