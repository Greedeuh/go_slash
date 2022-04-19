use diesel::{dsl::max, prelude::*};
use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde::Deserialize;
use serde_json::json;
use std::{cmp::Ordering, collections::HashMap};

use crate::{
    models::{
        features::Features,
        teams::{Team, TeamForOptUser},
        users::{read_or_write, should_be_logged_in_with, Right, User, UserTeam},
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
    user: Option<User>,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Template, (Status, Template)> {
    if !features.teams {
        return Err(AppError::Disable.into());
    }

    let user = should_be_logged_in_with(&Right::Read, user, &features)?;

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

    let right = read_or_write(&features, &Some(user.mail.clone()))?;

    Ok(Template::render(
        "teams",
        json!({ "teams": json!(teams).to_string(), "mail": &user.mail, "features": json!(features), "right": right  }),
    ))
}

#[delete("/go/teams/<team>")]
pub fn delete_team(
    team: String,
    user: User,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    if !features.teams {
        return Err(AppError::Disable.into());
    }

    if !user.is_admin {
        return Err(AppError::Unauthorized.into());
    }

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
    if !features.teams {
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
            is_accepted: user.is_admin,
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
    if !features.teams {
        return Err(AppError::Disable.into());
    }

    if !user.is_admin {
        return Err(AppError::Unauthorized.into());
    }

    let conn = pool.get().map_err(AppError::from)?;
    diesel::update(teams::table.find(team))
        .set(&data.into_inner())
        .execute(&conn)
        .map_err(AppError::from)?;

    Ok(Status::Ok)
}

#[put("/go/user/teams/ranks", data = "<team_ranks>")]
pub fn put_user_team_ranks(
    team_ranks: Json<HashMap<String, u16>>,
    user: Option<User>,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Template)> {
    if !features.teams {
        return Err(AppError::Disable.into());
    }

    let user = should_be_logged_in_with(&Right::Read, user, &features)?;

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
