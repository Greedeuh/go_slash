use std::collections::HashMap;
use std::str::FromStr;

use diesel::prelude::*;
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::models::settings::Features;
use crate::models::teams::{Team, TeamCapability};
use crate::models::users::{Capability, User, UserTeam, SAFE_USER_COLUMNS};
use crate::models::AppError;
use crate::schema::users::dsl;
use crate::schema::{teams, users_teams};
use crate::DbPool;

lazy_static! {
   pub static ref MAIL_REGEX: Regex =
        Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#,).unwrap();
}

#[derive(Deserialize)]
pub struct UserTeamLink {
    pub rank: i16,
}

#[post("/go/user/teams", data = "<team_user_link>")]
pub fn join_global_team(
    user: User,
    team_user_link: Json<UserTeamLink>,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    join_team("".to_string(), team_user_link, user, features, pool)
}

#[post("/go/user/teams/<slug>", data = "<team_user_link>")]
pub fn join_team(
    slug: String,
    team_user_link: Json<UserTeamLink>,
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    if !features.login.any() {
        return Err(AppError::Disable.into());
    }

    user.should_have_capability(Capability::UsersTeamsWrite)?;

    let conn = pool.get().map_err(AppError::from)?;
    let team: Option<Team> = teams::table
        .find(&slug)
        .first(&conn)
        .optional()
        .map_err(AppError::from)?;

    let team = if let Some(team) = team {
        team
    } else {
        return Err((Status::NotFound, json!({"error": "Team not found"})));
    };

    diesel::insert_into(users_teams::table)
        .values(UserTeam {
            user_mail: user.mail,
            team_slug: slug,
            capabilities: vec![TeamCapability::ShortcutsWrite],
            is_accepted: !team.is_private,
            rank: team_user_link.rank,
        })
        .execute(&conn)
        .map_err(AppError::from)?;

    Ok(Status::Created)
}

#[delete("/go/user/teams")]
pub fn leave_global_team(
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    leave_team("".to_string(), user, features, pool)
}

#[delete("/go/user/teams/<slug>")]
pub fn leave_team(
    slug: String,
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    if !features.login.any() {
        return Err(AppError::Disable.into());
    }

    user.should_have_capability(Capability::UsersTeamsWrite)?;

    let conn = pool.get().map_err(AppError::from)?;
    diesel::delete(users_teams::table)
        .filter(
            users_teams::user_mail
                .eq(&user.mail)
                .and(users_teams::team_slug.eq(&slug)),
        )
        .execute(&conn)
        .map_err(AppError::from)?;

    Ok(Status::Ok)
}

#[put("/go/user/teams/ranks", data = "<team_ranks>")]
pub fn put_user_team_ranks(
    team_ranks: Json<HashMap<String, u16>>,
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    if !features.teams {
        return Err(AppError::Disable.into());
    }

    user.should_have_capability(Capability::UsersTeamsWrite)?;

    let conn = pool.get().map_err(AppError::from)?;
    conn.transaction::<_, diesel::result::Error, _>(|| {
        for (slug, rank) in team_ranks.into_inner() {
            match diesel::update(users_teams::table.find((&user.mail, &slug)))
                .set(users_teams::rank.eq(rank as i16))
                .execute(&conn)
            {
                Ok(_) => (),
                Err(e) => {
                    error!(
                        "Team rank update failed for {} rank {}, rollback transaction: {:?}",
                        slug, rank, e
                    );
                    return Err(diesel::result::Error::RollbackTransaction);
                }
            }
        }
        Ok(())
    })
    .map_err(AppError::from)?;

    Ok(Status::Ok)
}

#[get("/go/users")]
pub fn list_users(
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Template, (Status, Value)> {
    if !features.login.any() {
        return Err(AppError::Disable.into());
    }

    let conn = pool.get().map_err(AppError::from)?;
    let users = dsl::users
        .select(SAFE_USER_COLUMNS)
        .order_by(dsl::mail)
        .load::<User>(&conn)
        .map_err(AppError::from)?;

    Ok(Template::render(
        "users",
        json!({
            "mail":  &user.mail,
            "features": json!(features),
            "context": json!({
                "users": users
            }).to_string()
        }),
    ))
}

#[put("/go/users/<mail>/capabilities/<capability>")]
pub fn put_user_capability(
    mail: String,
    capability: String,
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    let capability = Capability::from_str(&capability).map_err(|_| AppError::BadRequest)?;

    let conn = pool.get().map_err(AppError::from)?;

    if !features.login.any() {
        return Err(AppError::Disable.into());
    }

    user.should_have_capability(Capability::UsersAdmin)?;

    let user: User = dsl::users
        .select(SAFE_USER_COLUMNS)
        .find(&mail)
        .first(&conn)
        .map_err(AppError::from)?;

    let mut capabilities = user.capabilities;
    if !capabilities.contains(&capability) {
        capabilities.push(capability);
        diesel::update(dsl::users.find(&mail))
            .set(dsl::capabilities.eq(capabilities))
            .execute(&conn)
            .map_err(AppError::from)?;
    } else {
        warn!("User {} already has capability {}", mail, capability);
    }

    Ok(Status::Ok)
}

#[delete("/go/users/<mail>/capabilities/<capability>")]
pub fn delete_user_capability(
    mail: String,
    capability: String,
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    let capability = Capability::from_str(&capability).map_err(|_| AppError::BadRequest)?;

    let conn = pool.get().map_err(AppError::from)?;

    if !features.login.any() {
        return Err(AppError::Disable.into());
    }

    user.should_have_capability(Capability::UsersAdmin)?;

    let user: User = dsl::users
        .select(SAFE_USER_COLUMNS)
        .find(&mail)
        .first(&conn)
        .map_err(AppError::from)?;

    let mut capabilities = user.capabilities;
    if capabilities.contains(&capability) {
        capabilities.retain(|&c| c != capability);
        diesel::update(dsl::users.find(&mail))
            .set(dsl::capabilities.eq(capabilities))
            .execute(&conn)
            .map_err(AppError::from)?;
    } else {
        warn!(
            "User {} already does not have capability {}",
            mail, capability
        );
    }

    Ok(Status::Ok)
}
