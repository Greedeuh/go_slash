use diesel::prelude::*;
use go_web::{models::shortcuts::Shortcut, schema::shortcuts};

#[allow(dead_code)]
pub fn get_shortcut(shortcut: &str, conn: &SqliteConnection) -> Shortcut {
    shortcuts::table.find(shortcut).first(conn).unwrap()
}
