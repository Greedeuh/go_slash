use diesel::{prelude::*, Insertable};
use serde::Serialize;

use crate::models::AppError;
use crate::schema::shortcuts;
use crate::DbConn;

pub type AllColumns = (shortcuts::shortcut, shortcuts::team_slug, shortcuts::url);

pub const SHORTCUT_COLUMNS: AllColumns =
    (shortcuts::shortcut, shortcuts::team_slug, shortcuts::url);

#[derive(Queryable, Serialize, PartialEq, Eq, Debug)]
pub struct Shortcut {
    pub shortcut: String,
    pub team_slug: String,
    pub url: String,
}

#[derive(Insertable)]
#[diesel(table_name = shortcuts)]
pub struct NewShortcut {
    pub shortcut: String,
    pub url: String,
    pub team_slug: String,
}

#[derive(AsChangeset)]
#[diesel(table_name = shortcuts)]
pub struct UpdatableShortcut {
    pub url: String,
    pub team_slug: String,
}

pub fn sorted(conn: &mut DbConn) -> Result<Vec<Shortcut>, AppError> {
    use crate::schema::shortcuts::dsl::*;

    Ok(shortcuts.order(shortcut.asc()).load::<Shortcut>(conn)?)
}
