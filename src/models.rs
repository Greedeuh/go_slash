use std::collections::HashMap;

pub struct Entries {
    map: HashMap<String, Shortcut>,
}

impl Entries {
    pub fn new(map: HashMap<Shortcut, ShortcutUrl>) -> Self {
        Self { map }
    }

    pub fn find(&self, key: &str) -> Option<ShortcutUrl> {
        self.map.get(key).cloned()
    }
}

pub type ShortcutUrl = String;
pub type Shortcut = String;
