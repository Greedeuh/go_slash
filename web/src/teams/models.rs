use diesel::{deserialize, dsl::count, prelude::*, serialize, Identifiable};
use serde::{Deserialize, Serialize};
use std::{io::Write, str::FromStr};
use strum_macros::{Display, EnumString};

use crate::{
    errors::{
        AppError,
    },
    users::{User, UserTeam},
    schema::{teams, users_teams},
    DbConn,
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

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct TeamWithUsers {
    #[serde(flatten)]
    pub team: Team,
    pub user_links: Vec<UserTeam>,
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

#[derive(Queryable, Serialize)]
pub struct TeamForUserIfSome {
    #[serde(flatten)]
    pub team: Team,
    pub user_link: Option<UserTeam>,
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

pub fn teams_with_shortcut_write(user: &User, conn: &mut DbConn) -> Result<Vec<Team>, AppError> {
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
