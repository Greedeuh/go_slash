use rocket::State;
use std::{collections::HashMap, sync::Mutex};

use crate::models::features::Features;
use crate::schema::users;
use crate::{guards::SessionId, models::AppError};

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub mail: String,
    pub pwd: String,
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

pub fn should_be_logged_in_if_features(
    right: &Right,
    session_id: &Option<SessionId>,
    sessions: &State<Sessions>,
    features: &Features,
) -> Result<Option<String>, AppError> {
    if features.login.simple {
        match right {
            Right::Admin => should_be_logged_in(session_id, sessions),
            Right::Read if features.login.read_private => should_be_logged_in(session_id, sessions),
            Right::Write if features.login.write_private => {
                should_be_logged_in(session_id, sessions)
            }
            _ => match session_id {
                Some(session_id) => Ok(sessions.is_logged_in(&session_id.0)),
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
) -> Result<Option<String>, AppError> {
    if let Some(session_id) = session_id {
        match sessions.is_logged_in(&session_id.0) {
            None => {
                error!("Wrong session_id.");
                Err(AppError::Unauthorized)
            }
            Some(mail) => Ok(Some(mail)),
        }
    } else {
        Err(AppError::Unauthorized)
    }
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
