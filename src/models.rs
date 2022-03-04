use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct Entries {
    map: Arc<Mutex<HashMap<Shortcut, ShortcutUrl>>>,
}

impl Entries {
    pub fn new(map: HashMap<Shortcut, ShortcutUrl>) -> Self {
        Self {
            map: Arc::new(Mutex::new(map)),
        }
    }

    pub fn empty() -> Self {
        Self {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn all(&self) -> HashMap<String, String> {
        let map = Arc::clone(&self.map);
        let map = match map.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        map.clone()
    }

    pub fn find(&self, key: &str) -> Option<ShortcutUrl> {
        match Arc::clone(&self.map).lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
        .get(key)
        .cloned()
    }

    pub fn put(&self, key: &str, url: ShortcutUrl) {
        let map = Arc::clone(&self.map);
        let mut map = match map.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        map.insert(key.to_owned(), url);
    }

    pub fn delete(&self, key: &str) {
        let map = Arc::clone(&self.map);
        let mut map = match map.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        map.remove(key);
    }
}

impl<'a> From<&'a str> for Entries {
    fn from(shortcuts: &'a str) -> Self {
        let shortcuts = shortcuts
            .lines()
            .map(|line| {
                let line = line.replace(' ', "");
                let (key, value) = line
                    .split_once(':')
                    .expect("launch_with shortcuts failed parsing");
                (key.to_owned(), value.to_owned())
            })
            .collect();

        Self::new(shortcuts)
    }
}

pub type ShortcutUrl = String;
pub type Shortcut = String;
