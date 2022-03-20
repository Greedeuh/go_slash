use rustbreak::{deser::Yaml, PathDatabase};
use serde::Serialize;
use std::collections::HashMap;

use crate::models::AppError;
use crate::schema::shortcuts;

#[derive(Queryable, Serialize)]
pub struct Shortcut {
    pub shortcut: String,
    pub url: String,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "shortcuts"]
pub struct NewShortcut {
    pub shortcut: String,
    pub url: String,
}

pub trait Shortcuts {
    fn all(&self) -> Result<HashMap<String, String>, AppError>;
    fn sorted(&self) -> Result<Vec<(String, String)>, AppError>;
    fn find(&self, key: &str) -> Result<Option<String>, AppError>;
    fn put(&self, key: &str, url: String) -> Result<(), AppError>;
    fn delete(&self, key: &str) -> Result<(), AppError>;
}

pub struct Entries {
    db: PathDatabase<HashMap<String, String>, Yaml>,
}

impl Entries {
    pub fn from_path(path: &str) -> Self {
        let db: PathDatabase<HashMap<String, String>, Yaml> =
            PathDatabase::load_from_path_or_else(path.into(), HashMap::new).unwrap();
        Self { db }
    }
}

impl Shortcuts for Entries {
    fn all(&self) -> Result<HashMap<String, String>, AppError> {
        Ok(self.db.borrow_data()?.clone())
    }

    fn sorted(&self) -> Result<Vec<(String, String)>, AppError> {
        let mut all_shortcuts = self
            .all()?
            .into_iter()
            .map(|(shortcut, url)| (shortcut, url))
            .collect::<Vec<_>>();

        all_shortcuts.sort_by(|(shortcut_1, _), (shortcut_2, _)| shortcut_1.cmp(shortcut_2));

        Ok(all_shortcuts)
    }

    fn find(&self, key: &str) -> Result<Option<String>, AppError> {
        Ok(self.db.borrow_data()?.get(key).cloned())
    }

    fn put(&self, key: &str, url: String) -> Result<(), AppError> {
        {
            let mut db = self.db.borrow_data_mut()?;
            db.insert(key.to_owned(), url);
        }
        self.db.save()?;
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), AppError> {
        {
            let mut db = self.db.borrow_data_mut()?;
            db.remove(key);
        }
        self.db.save()?;
        Ok(())
    }
}
