mod controller;

pub use controller::*;

use diesel::{deserialize, dsl::{count, max}, prelude::*, serialize, Identifiable};
use serde::{Deserialize, Serialize};
use std::{io::Write, str::FromStr};
use strum_macros::{Display, EnumString};

use crate::{
    errors::AppError, schema::{        teams::{self, dsl}
    , users_teams}, users::{ User, UserTeam}, DbConn
};

pub type AllColumns = (
    teams::slug,
    teams::title,
    teams::is_private,
    teams::is_accepted,
);

pub const TEAM_COLUMNS: AllColumns = (
    teams::slug,
    teams::title,
    teams::is_private,
    teams::is_accepted,
);

#[derive(Insertable, Queryable, Serialize, Identifiable, Debug, PartialEq, Eq)]
#[diesel(table_name = teams, primary_key(slug))]
pub struct Team {
    pub slug: String,
    pub title: String,
    pub is_private: bool,
    pub is_accepted: bool,
}

impl Team {
    pub fn create(NewTeam{
        slug,
        title,
        is_private,
    }: NewTeam, user: &User, conn: &mut DbConn) -> Result<(Team, UserTeam), AppError> {
        let require_validation = user.can_create_teams()?;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let team = Team {
                slug: slug.clone(),
                is_accepted: !require_validation,
                title,
                is_private,
            };
            let team = db::insert(conn, team)?;
    
            let previous_rank = (db::next_user_team_rank(&user.mail, conn)?)
            .unwrap_or(0);
    
            let user_team = UserTeam {
                user_mail: user.mail.to_string(),
                team_slug: slug,
                capabilities: TeamCapability::all(),
                is_accepted: true,
                rank: previous_rank as i16 + 1,
            };
            let user_team = db::add_user(conn, user_team)?;
            Ok((team,user_team))
        })
        .map_err(AppError::from)
    }

    pub fn delete(slug: &str, user: &User, conn: &mut DbConn) -> Result<(), AppError> {
        user.can_write_team(slug, conn)?;
        db::delete(slug, conn).map_err(AppError::from).map(|_| ())
    }

    pub fn update(patchable_team: PatchableTeam, slug: &str, user: &User, conn: &mut DbConn) -> Result<Team, AppError> {
        user.can_write_team(slug, conn)?;
        
        if patchable_team.is_accepted.is_some() {
           user.can_accept_teams()?;
        }

        db::update(patchable_team, slug, conn).map_err(AppError::from)
    }

    pub fn find(slug: &str, user: &User, conn: &mut DbConn) -> Result<Option<Team>, AppError> {
        let team = db::find_by_slug(slug, conn).map_err(AppError::from)?;
        
        let team = if let Some(team) = team {
            team
        } else {
            return Ok(None);
        };

        user.can_read_team_shortcuts(&team, conn)?;
        
        Ok(Some(team))
    }

    pub fn all_with_shortcut_write(user: &User, conn: &mut DbConn) -> Result<Vec<Team>, AppError> {
        db::find_all_with_shortcut_write(&user.mail, conn).map_err(AppError::from)
    }

    pub fn all_with_user_link(mail: &str, conn: &mut DbConn) -> Result<Vec<TeamForOptUser>, AppError> {
        db::find_all_with_user_link(mail, conn).map_err(AppError::from)
    }

    pub fn with_all_user_links(slug: &str, user: &User, conn: &mut DbConn) -> Result<Option<TeamWithUserLinks>, AppError> {
        let team = db::find_by_slug(slug, conn).map_err(AppError::from)?;

        let team: Team = if let Some(team) = team {
            team
        } else {
            return Ok(None);
        };

        let user_links = db::find_user_links_for_team(&team.slug, conn).map_err(AppError::from)?;

        match user.can_read_users_teams(&team, conn) {
            Ok(_) => (),
            Err(AppError::Unauthorized) => {
                // don't say it exists but you can't see it, just say it doesn't exist for the user
                return Ok(None);
            },
            Err(e) => return Err(e),
        }

        Ok(Some(TeamWithUserLinks {
            team,
            user_links
        }))
    }

    pub fn kick_user(
        slug: &str,
        mail: &str,
        user: &User,
        conn: &mut DbConn,
    ) -> Result<usize, AppError> {
        let is_self_kick = user.mail == mail;
        if !is_self_kick {
            user.can_write_team(slug, conn)?;
        }

        db::remove_user_from_team(mail, slug, conn).map_err(AppError::from)
    }

    pub fn add_user_capability(
        mail: &str,
        team_slug: &str,
        capability: TeamCapability,
        user: &User,
        conn: &mut DbConn,
    ) -> Result<(), AppError> {
        user.can_write_team(team_slug, conn)?;

        let user_link: UserTeam = db::find_user_team_link(mail, team_slug, conn).map_err(AppError::from)?;

        let mut capabilities = user_link.capabilities;
        if !capabilities.contains(&capability) {
            capabilities.push(capability);
            db::update_user_capabilities(mail, team_slug, capabilities, conn).map_err(AppError::from)?;
        } else {
            warn!(
                "User {} already has capability {} on team {}",
                mail, capability, team_slug
            );
        }
        
        Ok(())
    }

    pub fn remove_user_capability(
        mail: &str,
        team_slug: &str,
        capability: TeamCapability,
        user: &User,
        conn: &mut DbConn,
    ) -> Result<(), AppError> {
        user.can_write_team(team_slug, conn)?;

        let user_link: UserTeam = db::find_user_team_link(mail, team_slug, conn).map_err(AppError::from)?;

        let mut capabilities = user_link.capabilities;
        if capabilities.contains(&capability) {
            capabilities.retain(|c| *c != capability);
            db::update_user_capabilities(mail, team_slug, capabilities, conn).map_err(AppError::from)?;
        } else {
            warn!(
                "User {} already has capability no {} on team {}",
                mail, capability, team_slug
            );
        }
        
        Ok(())
    }

    pub fn set_acceptation_user(
        mail: &str,
        team_slug: &str,
        acceptation: &bool,
        user: &User,
        conn: &mut DbConn,
    ) -> Result<(), AppError> {
        user.can_write_team(team_slug, conn)?;

        db::update_user_acceptance(mail, team_slug, *acceptation, conn).map_err(AppError::from).map(|_| ())
    }
}

mod db {
    use super::*;

    pub fn insert(conn: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>, team: Team) -> Result<Team, diesel::result::Error> {
        diesel::insert_into(teams::table)
            .values(team)
            .get_result(conn)
    }

    pub fn next_user_team_rank(user_mail: &str, conn: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<Option<i16>, diesel::result::Error> {
        users_teams::table
            .select(max(users_teams::rank))
            .filter(users_teams::user_mail.eq(&user_mail))
            .first::<Option<i16>>(conn)
    }

    pub fn delete(slug: &str, conn: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::delete(teams::table.find(slug))
        .execute(conn)
    }

    pub fn update(patchable_team: PatchableTeam, slug: &str, conn: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<Team, diesel::result::Error> {
        diesel::update(teams::table.find(slug))
        .set(&patchable_team)
        .get_result(conn)
    }

    pub fn add_user(conn: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>, user_team: UserTeam) -> Result<UserTeam, diesel::result::Error> {
        diesel::insert_into(users_teams::table)
            .values(user_team)
            .get_result(conn)
    }

    pub fn find_by_slug(slug: &str, conn: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<Option<Team>, diesel::result::Error> {
        teams::table.find(slug)
            .first::<Team>(conn)
            .optional()
    }

    pub fn find_all_with_shortcut_write(user_mail: &str, conn: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<Vec<Team>, diesel::result::Error> {
        teams::table
            .inner_join(
                users_teams::table.on(teams::slug
                    .eq(users_teams::team_slug)
                    .and(users_teams::user_mail.eq(user_mail))
                    .and(
                        users_teams::capabilities
                            .contains(vec![TeamCapability::ShortcutsWrite.to_string()]),
                    )
                    .and(users_teams::is_accepted)),
            )
            .filter(teams::is_accepted)
            .select(TEAM_COLUMNS)
            .load::<Team>(conn)
    }

    pub fn find_all_with_user_link(mail: &str, conn: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<Vec<TeamForOptUser>, diesel::result::Error> {
        dsl::teams
            .left_join(
                users_teams::table.on(dsl::slug
                    .eq(users_teams::team_slug)
                    .and(users_teams::user_mail.eq(mail))),
            )
            .load(conn)
    }

    pub fn find_user_links_for_team(team_slug: &str, conn: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<Vec<UserTeam>, diesel::result::Error> {
        users_teams::table
            .filter(users_teams::team_slug.eq(team_slug))
            .load::<UserTeam>(conn)
    }

    pub fn remove_user_from_team(mail: &str, slug: &str, conn: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::delete(users_teams::table.find((mail, slug)))
            .execute(conn)
    }

    pub fn find_user_team_link(mail: &str, team_slug: &str, conn: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<UserTeam, diesel::result::Error> {
        users_teams::table
            .find((mail, team_slug))
            .first(conn)
    }

    pub fn update_user_capabilities(mail: &str, team_slug: &str, capabilities: Vec<TeamCapability>, conn: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::update(users_teams::table.find((mail, team_slug)))
            .set(users_teams::capabilities.eq(capabilities))
            .execute(conn)
    }

    pub fn update_user_acceptance(mail: &str, team_slug: &str, acceptation: bool, conn: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<usize, diesel::result::Error> {
        diesel::update(users_teams::table.find((mail, team_slug)))
            .set(users_teams::is_accepted.eq(acceptation))
            .execute(conn)
    }
}

#[derive(Deserialize)]
pub struct NewTeam {
    pub slug: String,
    pub title: String,
    pub is_private: bool,
}

#[derive(Deserialize, AsChangeset)]
#[diesel(table_name = teams)]
pub struct PatchableTeam {
    pub title: Option<String>,
    pub is_private: Option<bool>,
    pub is_accepted: Option<bool>,
}

#[derive(Queryable, Serialize)]
pub struct TeamForOptUser {
    #[serde(flatten)]
    pub team: Team,
    pub user_link: Option<UserTeam>,
}

#[derive(Queryable, Serialize)]
pub struct TeamForUser {
    #[serde(flatten)]
    pub team: Team,
    pub user_link: UserTeam,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct TeamWithUserLinks {
    #[serde(flatten)]
    pub team: Team,
    pub user_links: Vec<UserTeam>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    FromSqlRow,
    EnumString,
    AsExpression,
    Display,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum TeamCapability {
    ShortcutsWrite,
    TeamsWrite,
}

impl TeamCapability {
    pub fn all() -> Vec<TeamCapability> {
        vec![TeamCapability::ShortcutsWrite, TeamCapability::TeamsWrite]
    }
}

impl deserialize::FromSql<diesel::sql_types::Text, diesel::pg::Pg> for TeamCapability {
    fn from_sql(bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let s: String =
            deserialize::FromSql::<diesel::sql_types::Text, diesel::pg::Pg>::from_sql(bytes)?;
        let r = TeamCapability::from_str(&s)?;
        Ok(r)
    }
}

impl serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg> for TeamCapability
where
    String: serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg>,
{
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<diesel::pg::Pg>,
    ) -> std::result::Result<
        diesel::serialize::IsNull,
        std::boxed::Box<(dyn std::error::Error + std::marker::Send + std::marker::Sync + 'static)>,
    >

    {
        out.write_all(self.to_string().as_bytes())?;
        Ok(diesel::serialize::IsNull::No)
    }
}

pub fn user_should_have_team_capability(
    user: &User,
    team_slug: &str,
    conn: &mut DbConn,
    capability: TeamCapability,
) -> Result<(), AppError> {
    if users_teams::table
        .find((&user.mail, &team_slug))
        .filter(users_teams::capabilities.contains(vec![capability]))
        .filter(users_teams::is_accepted)
        .select(count(users_teams::user_mail))
        .first::<i64>(conn)
        .map_err(AppError::from)?
        == 0
    {
        error!(
            "User {} miss capability or team capability ShortcutsWrite on team {}",
            user.mail, team_slug
        );
        Err(AppError::Unauthorized)
    } else {
        Ok(())
    }
}
