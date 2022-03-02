use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct Entries {
    map: Arc<Mutex<HashMap<String, Shortcut>>>,
}

impl Entries {
    pub fn new(map: HashMap<Shortcut, ShortcutUrl>) -> Self {
        Self {
            map: Arc::new(Mutex::new(map)),
        }
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
}

pub type ShortcutUrl = String;
pub type Shortcut = String;
