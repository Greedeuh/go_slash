use std::collections::HashMap;

use diesel::dsl::count;
use diesel::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sha256::digest;
use uuid::Uuid;

use crate::models::features::Features;
use crate::models::teams::{Team, TeamCapability};
use crate::models::users::{Capability, User, UserTeam, SAFE_USER_COLUMNS};
use crate::schema::users::dsl;
use crate::schema::{teams, users_teams};
use crate::DbPool;
use crate::{
    models::{users::Sessions, AppError},
    AppConfig,
};

lazy_static! {
    static ref MAIL_REGEX: Regex =
        Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#,).unwrap();
}

#[get("/go/login")]
pub fn login(conf: &State<AppConfig>, features: Features) -> Result<Template, (Status, Template)> {
    let mut context: Map<String, Value> = Map::new();
    if !features.login.simple {
        return Err(AppError::Disable.into());
    } else {
        context.insert(
            "simple_salt".to_string(),
            Value::String(conf.simple_login_salt1.clone()),
        );
    }

    Ok(Template::render("login", json!(context)))
}

#[derive(Deserialize, Serialize)]
pub struct LoginSuccessfull {
    pub token: String,
}

#[derive(Deserialize)]
pub struct Credentials {
    mail: String,
    pwd: String,
}

#[post("/go/login", data = "<credentials>")]
pub fn simple_login(
    credentials: Json<Credentials>,
    sessions: &State<Sessions>,
    config: &State<AppConfig>,
    features: Features,
    pool: &State<DbPool>,
) -> Result<Json<LoginSuccessfull>, (Status, Value)> {
    if !features.login.simple {
        return Err(AppError::Disable.into());
    }

    let credentials = credentials.into_inner();
    if !MAIL_REGEX.is_match(&credentials.mail) {
        return Err((Status::BadRequest, json!({"error": "Wrong mail format."})));
    }

    let pwd = digest(format!("{}{}", credentials.pwd, config.simple_login_salt2));

    let conn = pool.get().map_err(AppError::from)?;
    let mail_pwd_match: i64 = dsl::users
        .select(count(dsl::mail))
        .filter(dsl::pwd.eq(&pwd))
        .find(&credentials.mail)
        .first(&conn)
        .map_err(AppError::from)?;

    if mail_pwd_match != 1 {
        return Err((
            Status::Unauthorized,
            json!({ "error": "Wrong credentials." }),
        ));
    };

    let token = Uuid::new_v4();
    sessions.put(&token.to_simple().to_string(), &credentials.mail);

    Ok(Json(LoginSuccessfull {
        token: token.to_simple().to_string(),
    }))
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
    if !features.login.simple {
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
    if !features.login.simple {
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
    if !features.login.simple {
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
