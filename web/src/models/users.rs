use diesel::{prelude::*, Associations, Identifiable, Insertable};
use rocket::State;
use std::{collections::HashMap, sync::Mutex};

use crate::models::features::Features;
use crate::schema::users;
use crate::schema::users::dsl;
use crate::schema::users_teams;
use crate::DbConn;
use crate::{guards::SessionId, models::AppError};

#[derive(Insertable, Queryable, Identifiable)]
#[table_name = "users"]
#[primary_key(mail)]
pub struct User {
    pub mail: String,
    pub pwd: String,
    pub is_admin: bool,
}

#[derive(Identifiable, Queryable, Associations, Insertable, PartialEq, Debug, Serialize)]
#[belongs_to(Team, foreign_key = team_slug)]
#[belongs_to(User, foreign_key = user_mail)]
#[table_name = "users_teams"]
#[primary_key(user_mail, team_slug)]
pub struct UserTeam {
    #[serde(skip)]
    pub user_mail: String,
    #[serde(skip)]
    pub team_slug: String,
    pub is_admin: bool,
    pub is_accepted: bool,
}

#[derive(Default)]
pub struct Sessions {
    sessions: Mutex<HashMap<String, String>>,
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

pub enum Right {
    Admin,
    Read,
    Write,
}

pub fn read_or_write(features: &Features, user_mail: &Option<String>) -> Result<String, AppError> {
    let features = &features.login;

    Ok(
        if features.simple && features.write_private && user_mail.is_none() {
            "read".to_string()
        } else {
            "write".to_string()
        },
    )
}

pub fn should_be_logged_in_with(
    right: &Right,
    session_id: &Option<SessionId>,
    sessions: &State<Sessions>,
    features: &Features,
    conn: &DbConn,
) -> Result<User, AppError> {
    match should_be_logged_in_if_features_with(right, session_id, sessions, features, conn)? {
        Some(user) => Ok(user),
        None => Err(AppError::Unauthorized),
    }
}

pub fn should_be_logged_in_if_features_with(
    right: &Right,
    session_id: &Option<SessionId>,
    sessions: &State<Sessions>,
    features: &Features,
    conn: &DbConn,
) -> Result<Option<User>, AppError> {
    if features.login.simple {
        match right {
            Right::Admin => {
                let user = should_be_logged_in(session_id, sessions, conn)?;
                if user.is_admin {
                    Ok(Some(user))
                } else {
                    Err(AppError::Unauthorized)
                }
            }
            Right::Read if features.login.read_private => {
                should_be_logged_in(session_id, sessions, conn).map(Some)
            }
            Right::Write if features.login.write_private => {
                should_be_logged_in(session_id, sessions, conn).map(Some)
            }
            _ => match session_id {
                Some(session_id) => {
                    should_be_logged_in(&Some(session_id.clone()), sessions, conn).map(Some)
                }
                None => Ok(None),
            },
        }
    } else {
        Ok(None)
    }
}

fn should_be_logged_in(
    session_id: &Option<SessionId>,
    sessions: &State<Sessions>,
    conn: &DbConn,
) -> Result<User, AppError> {
    if let Some(session_id) = session_id {
        match sessions.is_logged_in(&session_id.0) {
            None => {
                error!("Wrong session_id.");
                Err(AppError::Unauthorized)
            }
            Some(mail) => Ok(dsl::users.find(&mail).first::<User>(conn)?),
        }
    } else {
        Err(AppError::Unauthorized)
    }
}
