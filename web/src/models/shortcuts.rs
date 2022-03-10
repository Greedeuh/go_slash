use rustbreak::{deser::Yaml, PathDatabase};
use std::collections::HashMap;

use crate::models::AppError;

pub struct Entries {
    db: PathDatabase<HashMap<Shortcut, ShortcutUrl>, Yaml>,
}

impl Entries {
    pub fn from_path(path: &str) -> Self {
        let db: PathDatabase<HashMap<Shortcut, ShortcutUrl>, Yaml> =
            PathDatabase::load_from_path_or_else(path.into(), HashMap::new).unwrap();
        Self { db }
    }

    pub fn all(&self) -> Result<HashMap<String, String>, AppError> {
        Ok(self.db.borrow_data()?.clone())
    }

    pub fn sorted(&self) -> Result<Vec<(String, String)>, AppError> {
        let mut all_shortcuts = self
            .all()?
            .into_iter()
            .map(|(shortcut, url)| (shortcut, url))
            .collect::<Vec<_>>();

        all_shortcuts.sort_by(|(shortcut_1, _), (shortcut_2, _)| shortcut_1.cmp(shortcut_2));

        Ok(all_shortcuts)
    }

    pub fn find(&self, key: &str) -> Result<Option<ShortcutUrl>, AppError> {
        Ok(self.db.borrow_data()?.get(key).cloned())
    }

    pub fn put(&self, key: &str, url: ShortcutUrl) -> Result<(), AppError> {
        let mut db = self.db.borrow_data_mut()?;
        db.insert(key.to_owned(), url);
        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<(), AppError> {
        let mut db = self.db.borrow_data_mut()?;
        db.remove(key);
        Ok(())
    }
}

pub type ShortcutUrl = String;
pub type Shortcut = String;
