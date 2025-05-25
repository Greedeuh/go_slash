use diesel::{
    prelude::*,
};
use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};
use std::{cmp::Ordering, str::FromStr};

use crate::{
    errors::{
        AppError,
    },
    shortcuts::Shortcut,
    teams::{
         Team, TeamCapability,
        TeamForOptUser,  TeamWithUserLinks, PatchableTeam, NewTeam
    },
    users::{ User, },
    schema::{
        shortcuts,

    },
    views::IndexContext,
    DbPool,
};

#[get("/go/teams")]
pub fn list_teams(user: User, pool: &State<DbPool>) -> Result<Template, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    let mut teams =  Team::all_with_user_link(&user.mail, &mut conn)?;

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


#[get("/go/teams/<slug>")]
pub fn show_team(
    slug: String,
    user: User,
    pool: &State<DbPool>,
) -> Result<Template, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    let team_with_user_links: Option<TeamWithUserLinks> = Team::with_all_user_links(&slug, &user, &mut conn)?;

    if team_with_user_links.is_none(){
        return Err(AppError::NotFound.into());
    }

    let shortcuts = shortcuts::table
        .filter(shortcuts::team_slug.eq(&slug))
        .order(shortcuts::shortcut.asc())
        .load::<Shortcut>(&mut conn)
        .map_err(AppError::from)?;


    Ok(Template::render(
        "index",
        json!({
            "mail": &user.mail,
            "context": json!(IndexContext {
                shortcut: None,
                shortcuts,
                team: team_with_user_links,
                teams: Team::all_with_shortcut_write(&user, &mut conn)?,
                user
            }).to_string()
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

    Team::delete(&slug, &user, &mut conn)?;

    Ok(Status::Ok)
}

#[post("/go/teams", data = "<new_team>")]
pub fn create_team(
    new_team: Json<NewTeam>,
    user: User,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;
    Team::create(new_team.into_inner(), &user, &mut conn)?;

    Ok(Status::Created)
}

#[patch("/go/teams/<team>", data = "<patchable_team>")]
pub fn patch_team(
    team: String,
    patchable_team: Json<PatchableTeam>,
    user: User,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    Team::update(patchable_team.into_inner(), &team, &user, &mut conn)?;

    Ok(Status::Ok)
}

#[delete("/go/teams/<slug>/users/<mail>")]
pub fn kick_user(
    slug: String,
    mail: String,
    user: User,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    Team::kick_user(&slug, &mail,&user, &mut conn)?;

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

    Team::add_user_capability(&mail, &team_slug, capability,&user, &mut conn)?;

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

    Team::remove_user_capability(&mail, &team_slug, capability, &user, &mut conn)?;

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

    Team::set_acceptation_user(&mail, &team_slug,&acceptation, &user, &mut conn)?;

    Ok(Status::Ok)
}
