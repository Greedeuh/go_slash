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
 pub fn create( NewTeam{
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
            let team = diesel::insert_into(teams::table)
                .values(team)
                .get_result(conn)?;
    
            let previous_rank = (users_teams::table
                .select(max(users_teams::rank))
                .filter(users_teams::user_mail.eq(&user.mail))
                .first::<Option<i16>>(conn)?)
            .unwrap_or(0);
    
            let user_team = UserTeam {
                user_mail: user.mail.to_string(),
                team_slug: slug,
                capabilities: TeamCapability::all(),
                is_accepted: true,
                rank: previous_rank as i16 + 1,
            };
            let user_team = diesel::insert_into(users_teams::table)
                .values(user_team)
                .get_result(conn)?;
            Ok((team,user_team))
        })
        .map_err(AppError::from)
    }

    pub fn delete(slug: &str, user:&User,conn: &mut DbConn) -> Result<(), AppError> {
        user.can_write_team(slug, conn)?;

        diesel::delete(teams::table.find(slug))
        .execute(conn)
        .map_err(AppError::from).map(|_| ())
    }

    pub fn update(patchable_team: PatchableTeam, slug: &str, user: &User, conn: &mut DbConn) -> Result<Team, AppError> {
        user.can_write_team(slug, conn)?;
        
        if patchable_team.is_accepted.is_some() {
           user.can_accept_teams()?;
        }

        diesel::update(teams::table.find(slug))
        .set(&patchable_team)
        .get_result(conn)
        .map_err(AppError::from)
    }

    pub fn find(slug: &str, user: &User, conn: &mut DbConn) -> Result<Option<Team>, AppError> {
       let team = teams::table.find(slug)
    
            .first::<Team>(conn)
            .optional()
            .map_err(AppError::from)?;
        
        let team = if let Some(team) = team {
            team
        } else{
            return Ok(None);
        };

        user.can_read_team_shortcuts(&team, conn)?;
        
        Ok(Some(team))
    }

    pub fn all_with_shortcut_write(user: &User, conn: &mut DbConn) -> Result<Vec<Team>, AppError> {
        teams::table
            .inner_join(
                users_teams::table.on(teams::slug
                    .eq(users_teams::team_slug)
                    .and(users_teams::user_mail.eq(&user.mail))
                    .and(
                        users_teams::capabilities
                            .contains(vec![TeamCapability::ShortcutsWrite.to_string()]),
                    )
                    .and(users_teams::is_accepted)),
            )
            .filter(teams::is_accepted)
            .select(TEAM_COLUMNS)
            .load::<Team>(conn)
            .map_err(AppError::from)
    }

    pub fn all_with_user_link(mail: &str, conn: &mut DbConn) -> Result<Vec<TeamForOptUser>, AppError> {
        dsl::teams
        .left_join(
            users_teams::table.on(dsl::slug
                .eq(users_teams::team_slug)
                .and(users_teams::user_mail.eq(mail))),
        )
        .load(conn)
        .map_err(AppError::from)
    }

    pub fn with_all_user_links(slug: &str, user: &User, conn: &mut DbConn)-> Result<Option<TeamWithUserLinks>, AppError> {
        let team = teams::table
        .find(&slug)
        .first( conn)
        .optional()
        .map_err(AppError::from)?;

        let team: Team = if let Some(team) = team {
            team
        } else {
            return Ok(None);
        };

        let user_links = UserTeam::belonging_to(&team)
        .load::<UserTeam>(conn)
        .map_err(AppError::from)?;


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
        if user.mail != mail {
            user.can_write_team(slug, conn)?;
        }

        diesel::delete(users_teams::table.find((mail, slug)))
        .execute(conn)
        .map_err(AppError::from)
    }

    pub fn add_user_capability(
        mail: &str,
        team_slug: &str,
        capability: TeamCapability,
        user: &User,
        conn: &mut DbConn,
    ) -> Result<(), AppError> {
        user.can_write_team(team_slug, conn)?;

        let user_link: UserTeam = users_teams::table
        .find((mail, team_slug))
        .first(conn)
        .map_err(AppError::from)?;

        let mut capabilities = user_link.capabilities;
        if !capabilities.contains(&capability) {
            capabilities.push(capability);
            diesel::update(users_teams::table.find((mail, team_slug)))
                .set(users_teams::capabilities.eq(capabilities))
                .execute(conn)
                .map_err(AppError::from)?;
        }else {
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

        let user_link: UserTeam = users_teams::table
        .find((mail, team_slug))
        .first(conn)
        .map_err(AppError::from)?;

        let mut capabilities = user_link.capabilities;
        if capabilities.contains(&capability) {
            capabilities.retain(|&c| c != capability);
            diesel::update(users_teams::table.find((mail, team_slug)))
                .set(users_teams::capabilities.eq(capabilities))
                .execute(conn)
                .map_err(AppError::from)?;
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

        diesel::update(users_teams::table.find((mail, team_slug)))
        .set(users_teams::is_accepted.eq(acceptation))
        .execute(conn)
        .map_err(AppError::from).map(|_| ())
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
