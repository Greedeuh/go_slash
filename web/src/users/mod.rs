mod controller;
mod sessions;

pub use controller::*;
use diesel::{dsl::{count}, prelude::*};
pub use sessions::*;

use crate::teams::{ Team, TeamCapability};
use crate::errors::AppError;
use crate::schema::users;
use crate::{schema::*, DbConn};
use diesel::{deserialize, serialize, Associations, Identifiable, Insertable, PgArrayExpressionMethods};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str::FromStr;
use std::vec;
use strum_macros::{Display, EnumString};

pub type SafeColumns = (users::mail, users::capabilities);

pub const SAFE_USER_COLUMNS: SafeColumns = (users::mail, users::capabilities);

#[derive(Queryable, Identifiable, Serialize, Debug, PartialEq)]
#[diesel(table_name = users, primary_key(mail))]
pub struct User {
    pub mail: String,
    pub capabilities: Vec<Capability>,
}

#[derive(Insertable, Queryable, Identifiable, Debug)]
#[diesel(table_name = users, primary_key(mail))]
pub struct UserWithPwd {
    pub mail: String,
    pub pwd: Option<String>,
    pub capabilities: Vec<Capability>,
}

#[derive(Identifiable, Queryable, Associations, Insertable, PartialEq, Debug, Serialize, Eq)]
#[diesel(table_name = users_teams, primary_key(user_mail, team_slug), belongs_to(Team, foreign_key = team_slug),belongs_to(User, foreign_key = user_mail))]
pub struct UserTeam {
    pub user_mail: String,
    pub team_slug: String,
    pub capabilities: Vec<TeamCapability>,
    pub is_accepted: bool,
    pub rank: i16,
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
pub enum Capability {
    Features,
    TeamsWrite,
    TeamsCreateWithValidation,
    UsersTeamsRead,
    UsersTeamsWrite,
    UsersAdmin,
}

type RequireValidation = bool;
impl User {
    pub fn can_create_teams(&self) ->  Result<RequireValidation, AppError>  {
        self.should_have_one_of_theses_capabilities(&[
            Capability::TeamsWrite,
            Capability::TeamsCreateWithValidation,
        ])?;

        Ok(!self.have_capability(Capability::TeamsWrite))
    }

    pub fn can_write_team(&self, team_slug: &str, conn: &mut DbConn) -> Result<(), AppError> {
        if self.have_capability(Capability::TeamsWrite) {
            return Ok(());
        } 
        
        self.user_should_have_team_capability(team_slug, conn, TeamCapability::TeamsWrite)
    }

    pub fn can_accept_teams(&self) -> Result<(), AppError> {
         self.should_have_capability(Capability::TeamsWrite) 
    }

    pub fn can_read_users_teams(&self, team: &Team, conn: &mut DbConn) -> Result<(), AppError> {
        if !team.is_private {
            return Ok(());
        }

        if self.have_capability(Capability::UsersTeamsRead) {
            return Ok(());
        }

        self.user_should_have_team(&team.slug, conn)
    }

    pub fn user_should_have_team_capability(
        &self,
        team_slug: &str,
        conn: &mut DbConn,
        capability: TeamCapability,
    ) -> Result<(), AppError> {
        if users_teams::table
            .find((&self.mail, &team_slug))
            .filter(users_teams::capabilities.contains(vec![capability]))
            .filter(users_teams::is_accepted)
            .select(count(users_teams::user_mail))
            .first::<i64>(conn)
            .map_err(AppError::from)?
            == 0
        {
            error!(
                "User {} miss capability or team capability {} on team {}",
                self.mail,capability, team_slug
            );
            Err(AppError::Unauthorized)
        } else {
            Ok(())
        }
    }

   pub fn user_should_have_team(
        &self,
        team_slug: &str,
        conn: &mut DbConn,
    ) -> Result<(), AppError> {
        if users_teams::table
            .find((&self.mail, &team_slug))
            .filter(users_teams::is_accepted)
            .select(count(users_teams::user_mail))
            .first::<i64>(conn)
            .map_err(AppError::from)?
            == 0
        {
            error!(
                "User {} miss team {}",
                self.mail, team_slug
            );
            Err(AppError::Unauthorized)
        } else {
            Ok(())
        }
    }


    pub fn fake_admin() -> Self {
        Self {
            mail: "fake_admin".to_string(),
            capabilities: Capability::all(),
        }
    }

   


    pub fn have_capability(&self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }

    pub fn should_have_capability(&self, capability: Capability) -> Result<(), AppError> {
        if self.have_capability(capability) {
            Ok(())
        } else {
            error!("User {} miss capability {}", self.mail, capability);
            Err(AppError::Unauthorized)
        }
    }

    pub fn should_have_one_of_theses_capabilities(
        &self,
        capabilities: &[Capability],
    ) -> Result<(), AppError> {
        if capabilities
            .iter()
            .any(|capability| self.capabilities.contains(capability))
        {
            Ok(())
        } else {
            error!(
                "User {} miss one of theses capabilities {:?}",
                self.mail, capabilities
            );
            Err(AppError::Unauthorized)
        }
    }

     pub fn should_have_all_theses_capabilities(
        &self,
        capabilities: &[Capability],
    ) -> Result<(), AppError> {
        if capabilities
            .iter()
            .all(|capability| self.capabilities.contains(capability))
        {
            Ok(())
        } else {
            error!(
                "User {} miss one of theses capabilities {:?}",
                self.mail, capabilities
            );
            Err(AppError::Unauthorized)
        }
    }
}

impl Capability {
    pub fn all() -> Vec<Capability> {
        vec![
            Capability::Features,
            Capability::TeamsWrite,
            Capability::TeamsCreateWithValidation,
            Capability::UsersTeamsRead,
            Capability::UsersTeamsWrite,
            Capability::UsersAdmin,
        ]
    }
}

impl deserialize::FromSql<diesel::sql_types::Text, diesel::pg::Pg> for Capability {
    fn from_sql(bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let s: String =
        deserialize::FromSql::<diesel::sql_types::Text, diesel::pg::Pg>::from_sql(bytes)?;
    let r = Capability::from_str(&s)?;
    Ok(r)
    }
}

impl serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg> for Capability
where
    String: serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg>,
{
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, diesel::pg::Pg>) -> serialize::Result {
        out.write_all(self.to_string().as_bytes())?;
        Ok(diesel::serialize::IsNull::No)
    }
}

