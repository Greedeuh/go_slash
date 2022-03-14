use std::{collections::HashMap, sync::Mutex};

use rustbreak::{deser::Yaml, PathDatabase};
use serde::{Deserialize, Serialize};

use crate::models::AppError;

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
    pub fn put(&self, mail: &str, token: &str) {
        let mut sessions = match self.sessions.lock() {
            Ok(s) => s,
            Err(s) => s.into_inner(),
        };
        sessions.insert(mail.to_string(), token.to_string());
    }
}
