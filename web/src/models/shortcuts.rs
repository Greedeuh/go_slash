use diesel::{prelude::*, Insertable};
use serde::Serialize;

use crate::models::AppError;
use crate::schema::shortcuts;
use crate::DbConn;

#[derive(Queryable, Serialize)]
pub struct Shortcut {
    pub shortcut: String,
    pub url: String,
}

#[derive(Insertable)]
#[table_name = "shortcuts"]
pub struct NewShortcut {
    pub shortcut: String,
    pub url: String,
}

pub fn sorted(conn: &DbConn) -> Result<Vec<Shortcut>, AppError> {
    use crate::schema::shortcuts::dsl::*;

    Ok(shortcuts.order(shortcut.asc()).load::<Shortcut>(conn)?)
}