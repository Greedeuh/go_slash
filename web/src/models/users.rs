use std::{collections::HashMap, sync::Mutex};

use rocket::State;
use rustbreak::{deser::Yaml, PathDatabase};
use serde::{Deserialize, Serialize};

use crate::{guards::SessionId, models::AppError, GlobalFeatures};

pub struct SimpleUsers {
    users: PathDatabase<HashMap<String, User>, Yaml>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct User {
    pub pwd: String,
}

impl SimpleUsers {
    pub fn from_path(path: &str) -> Self {
        let users: PathDatabase<HashMap<String, User>, Yaml> =
            PathDatabase::load_from_path_or_else(path.into(), HashMap::new).unwrap();
        Self { users }
    }

    pub fn get_matching_pwd(&self, mail: &str, pwd: &str) -> Result<Option<User>, AppError> {
        match self.users.borrow_data()?.get(mail) {
            Some(user) if user.pwd == pwd => Ok(Some(user.clone())),
            _ => Ok(None),
        }
    }
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
        sessions.insert(mail.to_string(), session_id.to_string());
    }

    pub fn is_logged_in(&self, session_id: &str) -> bool {
        let sessions = match self.sessions.lock() {
            Ok(s) => s,
            Err(s) => s.into_inner(),
        };

        sessions.contains_key(session_id)
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

pub fn should_be_logged_in(
    session_id: Option<SessionId>,
    sessions: &State<Sessions>,
    features: &State<GlobalFeatures>,
) -> Result<(), AppError> {
    if features.get()?.login.simple {
        if let Some(session_id) = session_id {
            if !sessions.is_logged_in(&session_id.0) {
                error!("Wrong session_id.");
                return Err(AppError::Unauthorized);
            }
        } else {
            return Err(AppError::Unauthorized);
        }
    }
    Ok(())
}
