use std::str::FromStr;

use diesel::{deserialize, serialize, Identifiable};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use crate::{models::users::UserTeam, schema::teams};

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
