use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use go_web::{models::shortcuts::NewShortcut, schema::shortcuts};

#[allow(dead_code)]
pub fn shortcut(shortcut: &str, url: &str, db_con: &SqliteConnection) {
    diesel::insert_into(shortcuts::table)
        .values(&NewShortcut {
            shortcut: shortcut.to_string(),
            url: url.to_string(),
        })
        .execute(db_con)
        .unwrap();
}
