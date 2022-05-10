use diesel::{deserialize, prelude::*, serialize, Identifiable};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::{Display, EnumString};

use crate::{
    models::{
        users::{Capability, User, UserTeam},
        AppError,
    },
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
#[table_name = "teams"]
#[primary_key(slug)]
pub struct Team {
    pub slug: String,
    pub title: String,
    pub is_private: bool,
    pub is_accepted: bool,
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
#[sql_type = "diesel::sql_types::Text"]
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
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
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
    fn to_sql<W>(
        &self,
        out: &mut diesel::serialize::Output<'_, W, diesel::pg::Pg>,
    ) -> std::result::Result<
        diesel::serialize::IsNull,
        std::boxed::Box<(dyn std::error::Error + std::marker::Send + std::marker::Sync + 'static)>,
    >
    where
        W: std::io::Write,
    {
        out.write_all(self.to_string().as_bytes())?;
        Ok(diesel::types::IsNull::No)
    }
}

pub fn teams_with_shortcut_write(user: &User, conn: &DbConn) -> Result<Vec<Team>, AppError> {
    if user.have_capability(Capability::ShortcutsWrite) {
        teams::table
            .select(TEAM_COLUMNS)
            .load::<Team>(conn)
            .map_err(AppError::from)
    } else {
        teams::table
            .inner_join(
                users_teams::table.on(teams::slug
                    .eq(users_teams::team_slug)
                    .and(users_teams::user_mail.eq(&user.mail))
                    .and(
                        users_teams::capabilities
                            .contains(vec![TeamCapability::ShortcutsWrite.to_string()]),
                    )),
            )
            .select(TEAM_COLUMNS)
            .load::<Team>(conn)
            .map_err(AppError::from)
    }
}
