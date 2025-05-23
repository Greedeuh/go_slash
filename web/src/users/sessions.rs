use std::{collections::HashMap, sync::Mutex};


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