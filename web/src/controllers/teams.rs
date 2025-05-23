use diesel::{
    dsl::{count, max},
    prelude::*,
};
use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{cmp::Ordering, str::FromStr};

use crate::{
    models::{
        shortcuts::Shortcut,
        teams::{
            teams_with_shortcut_write, user_should_have_team_capability, Team, TeamCapability,
            TeamForOptUser, TeamForUserIfSome, TeamWithUsers,
        },
        users::{Capability, User, UserTeam},
        AppError,
    },
    schema::{
        shortcuts,
        teams::{self, dsl},
        users_teams,
    },
    views::IndexContext,
    DbPool,
};

#[get("/go/teams")]
pub fn list_teams(user: User, pool: &State<DbPool>) -> Result<Template, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;
    let mut teams: Vec<TeamForOptUser> = dsl::teams
        .left_join(
            users_teams::table.on(dsl::slug
                .eq(users_teams::team_slug)
                .and(users_teams::user_mail.eq(&user.mail))),
        )
        .load(&mut conn)
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
            "capabilities": json!(user.capabilities).to_string(),
            "mail": user.mail
        }),
    ))
}

#[delete("/go/teams/<team>")]
pub fn delete_team(
    team: String,
    user: User,

    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    if !user.have_capability(Capability::TeamsWrite) {
        user_should_have_team_capability(&user, &team, &mut conn, TeamCapability::TeamsWrite)?;
    }

    diesel::delete(teams::table.find(team))
        .execute(&mut conn)
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

    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    user.should_have_one_of_theses_capabilities(&[
        Capability::TeamsWrite,
        Capability::TeamsWriteWithValidation,
    ])?;

    let mut conn = pool.get().map_err(AppError::from)?;
    conn.transaction::<_, diesel::result::Error, _>(|conn| {
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
            .execute(conn)?;

        let previous_rank = (users_teams::table
            .select(max(users_teams::rank))
            .filter(users_teams::user_mail.eq(&user.mail))
            .first::<Option<i16>>(conn)?)
        .unwrap_or(0);

        let user_team = UserTeam {
            user_mail: user.mail,
            team_slug: slug,
            capabilities: TeamCapability::all(),
            is_accepted: true,
            rank: previous_rank as i16 + 1,
        };
        diesel::insert_into(users_teams::table)
            .values(user_team)
            .execute(conn)?;
        Ok(())
    })
    .map_err(AppError::from)?;

    Ok(Status::Created)
}

#[derive(Deserialize, AsChangeset)]
#[diesel(table_name = teams)]
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

    diesel::update(teams::table.find(team))
        .set(&data.into_inner())
        .execute(&mut conn)
        .map_err(AppError::from)?;

    Ok(Status::Ok)
}

#[get("/go/teams/<slug>")]
pub fn show_team(
    slug: String,
    user: User,
    pool: &State<DbPool>,
) -> Result<Template, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    let team_with_user_if_some: Option<TeamForUserIfSome> = teams::table
        .find(&slug)
        .left_join(users_teams::table)
        .first(&mut conn)
        .optional()
        .map_err(AppError::from)?;

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
                teams: teams_with_shortcut_write(&user, &mut conn)?,
                user
            }).to_string()
        }),
    ))
}

#[delete("/go/teams/<team>/users/<mail>")]
pub fn kick_user(
    team: String,
    mail: String,
    user: User,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    // should be admin or (part of the team but can't change is_accepted)
    if !user.have_capability(Capability::TeamsWrite) {
        user_should_have_team_capability(&user, &team, &mut conn, TeamCapability::TeamsWrite)?;
    }

    diesel::delete(users_teams::table.find((mail, team)))
        .execute(&mut conn)
        .map_err(AppError::from)?;

    Ok(Status::Ok)
}

#[put("/go/teams/<team>/users/<mail>/capabilities/<capability>")]
pub fn put_user_link_capability(
    team: String,
    mail: String,
    capability: String,
    user: User,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    let capability = TeamCapability::from_str(&capability).map_err(|_| AppError::BadRequest)?;

    let mut conn = pool.get().map_err(AppError::from)?;

    if !user.have_capability(Capability::TeamsWrite) {
        user_should_have_team_capability(&user, &team, &mut conn, TeamCapability::TeamsWrite)?;
    }

    let user_link: UserTeam = users_teams::table
        .find((&mail, &team))
        .first(&mut conn)
        .map_err(AppError::from)?;

    let mut capabilities = user_link.capabilities;
    if !capabilities.contains(&capability) {
        capabilities.push(capability);
        diesel::update(users_teams::table.find((&mail, &team)))
            .set(users_teams::capabilities.eq(capabilities))
            .execute(&mut conn)
            .map_err(AppError::from)?;
    } else {
        warn!(
            "User {} already has capability {} on team {}",
            mail, capability, team
        );
    }

    Ok(Status::Ok)
}

#[delete("/go/teams/<team>/users/<mail>/capabilities/<capability>")]
pub fn delete_user_link_capability(
    team: String,
    mail: String,
    capability: String,
    user: User,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    let capability = TeamCapability::from_str(&capability).map_err(|_| AppError::BadRequest)?;

    let mut conn = pool.get().map_err(AppError::from)?;

    if !user.have_capability(Capability::TeamsWrite) {
        user_should_have_team_capability(&user, &team, &mut conn, TeamCapability::TeamsWrite)?;
    }

    let user_link: UserTeam = users_teams::table
        .find((&mail, &team))
        .first(&mut conn)
        .map_err(AppError::from)?;

    let mut capabilities = user_link.capabilities;
    if capabilities.contains(&capability) {
        capabilities.retain(|&c| c != capability);
        diesel::update(users_teams::table.find((&mail, &team)))
            .set(users_teams::capabilities.eq(capabilities))
            .execute(&mut conn)
            .map_err(AppError::from)?;
    } else {
        warn!(
            "User {} already has capability no {} on team {}",
            mail, capability, team
        );
    }

    Ok(Status::Ok)
}

#[put("/go/teams/<team>/users/<mail>/is_accepted/<value>")]
pub fn put_user_team_acceptation(
    team: String,
    mail: String,
    value: bool,
    user: User,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    let mut conn = pool.get().map_err(AppError::from)?;

    if !user.have_capability(Capability::TeamsWrite) {
        user_should_have_team_capability(&user, &team, &mut conn, TeamCapability::TeamsWrite)?;
    }

    diesel::update(users_teams::table.find((&mail, &team)))
        .set(users_teams::is_accepted.eq(value))
        .execute(&mut conn)
        .map_err(AppError::from)?;

    Ok(Status::Ok)
}
