use crate::models::teams::TeamCapability;
use crate::models::AppError;
use crate::schema::users;
use crate::schema::users_teams;
use diesel::{deserialize, serialize, Associations, Identifiable, Insertable};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::vec;
use std::{collections::HashMap, sync::Mutex};
use strum_macros::{Display, EnumString};

pub type SafeColumns = (users::mail, users::capabilities);

pub const SAFE_USER_COLUMNS: SafeColumns = (users::mail, users::capabilities);

#[derive(Queryable, Identifiable, Serialize, Debug, PartialEq)]
#[table_name = "users"]
#[primary_key(mail)]
pub struct User {
    pub mail: String,
    pub capabilities: Vec<Capability>,
}

#[derive(Insertable, Queryable, Identifiable, Debug)]
#[table_name = "users"]
#[primary_key(mail)]
pub struct UserWithPwd {
    pub mail: String,
    pub pwd: Option<String>,
    pub capabilities: Vec<Capability>,
}

#[derive(Identifiable, Queryable, Associations, Insertable, PartialEq, Debug, Serialize, Eq)]
#[belongs_to(Team, foreign_key = team_slug)]
#[belongs_to(User, foreign_key = user_mail)]
#[table_name = "users_teams"]
#[primary_key(user_mail, team_slug)]
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
#[sql_type = "diesel::sql_types::Text"]
pub enum Capability {
    Features,
    TeamsWrite,
    TeamsWriteWithValidation,
    UsersTeamsRead,
    UsersTeamsWrite,
    UsersAdmin,
}

#[derive(Default)]
pub struct Sessions {
    sessions: Mutex<HashMap<String, String>>,
}

impl User {
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
}

impl Capability {
    pub fn all() -> Vec<Capability> {
        vec![
            Capability::Features,
            Capability::TeamsWrite,
            Capability::TeamsWriteWithValidation,
            Capability::UsersTeamsRead,
            Capability::UsersTeamsWrite,
            Capability::UsersAdmin,
        ]
    }
}

impl deserialize::FromSql<diesel::sql_types::Text, diesel::pg::Pg> for Capability {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
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

impl Sessions {
    pub fn put(&self, session_id: &str, mail: &str) {
        let mut sessions = match self.sessions.lock() {
            Ok(s) => s,
            Err(s) => s.into_inner(),
        };
        sessions.insert(session_id.to_string(), mail.to_string());
    }

    pub fn is_logged_in(&self, session_id: &str) -> Option<String> {
        let sessions = match self.sessions.lock() {
            Ok(s) => s,
            Err(s) => s.into_inner(),
        };

        sessions.get(session_id).cloned()
    }
}

impl From<&str> for Sessions {
    fn from(sessions: &str) -> Self {
        if sessions.is_empty() {
            return Sessions::default();
        }
        Self {
            sessions: serde_yaml::from_str(sessions).unwrap(),
        }
    }
}
