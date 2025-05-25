mod controller;

pub use controller::*;

use diesel::{prelude::*, Insertable};
use serde::Serialize;

use crate::errors::AppError;
use crate::schema::shortcuts;
use crate::teams::Team;
use crate::users::User;
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

impl Shortcut {
    pub fn of_team(team: &Team, user: &User, conn: &mut DbConn) -> Result<Vec<Shortcut>, AppError> {
      match user.can_read_team_shortcuts(team, conn) {
        Ok(_) => (),
        Err(AppError::Unauthorized) => return Ok(vec![]),  
        Err(e) => return Err(e),  
      }

      shortcuts::table
        .filter(shortcuts::team_slug.eq(&team.slug))
        .order(shortcuts::shortcut.asc())
        .load::<Shortcut>(conn)
        .map_err(AppError::from)
    }
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
