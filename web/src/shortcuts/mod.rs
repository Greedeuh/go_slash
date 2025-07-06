mod controller;

pub use controller::*;

use diesel::{prelude::*, Insertable};
use serde::Serialize;

use crate::errors::AppError;
use crate::schema::{shortcuts, users_teams};
use crate::teams::Team;
use crate::users::User;
use crate::DbConn;
use crate::schema::shortcuts::dsl::*;

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

        db::of_team(team, conn).map_err(AppError::from)
    }

    pub fn sorted(user: &User, conn: &mut DbConn) -> Result<Vec<Shortcut>, AppError> {
        db::sorted(user, conn).map_err(AppError::from)
    }

    pub fn first(
        name: &str,
        conn: &mut DbConn,
        user: &User,
    ) -> Result<Option<Shortcut>, AppError> {
        db::first(name, user, conn).map_err(AppError::from)
    }

    pub fn upsert(
        new_shortcut: NewShortcut,
        user: &User,
        conn: &mut DbConn,
    ) -> Result<Shortcut, AppError> {
        let team = Team::find(&new_shortcut.team_slug, user, conn)
            .map_err(AppError::from)?;

        let team = if let Some(team) = team {
            team
        } else {
            return Err(AppError::NotFound);
        };

        user.can_write_team_shortcuts(&team, conn)?;
        
        db::upsert(new_shortcut, conn).map_err(AppError::from)
    }

    pub fn delete(
        shortcut_name: &str,
        team_slug_name: &str,
        user: &User,
        conn: &mut DbConn,
    ) -> Result<(), AppError> {
        let team = Team::find(team_slug_name, user, conn)
            .map_err(AppError::from)?;

        let team = if let Some(team) = team {
            team
        } else {
            return Err(AppError::NotFound);
        };

        user.can_write_team_shortcuts(&team, conn)?;

        db::delete(shortcut_name, team_slug_name, conn).map_err(AppError::from)
    }
}

mod db {
    use super::*;

    pub fn of_team(team: &Team, conn: &mut DbConn) -> Result<Vec<Shortcut>, diesel::result::Error> {
        shortcuts::table
            .filter(shortcuts::team_slug.eq(&team.slug))
            .order(shortcuts::shortcut.asc())
            .load::<Shortcut>(conn)
    }

    pub fn sorted(user: &User, conn: &mut DbConn) -> Result<Vec<Shortcut>, diesel::result::Error> {
        shortcuts
            .inner_join(
                users_teams::table.on(team_slug
                    .eq(users_teams::team_slug)
                    .and(users_teams::user_mail.eq(&user.mail))
                    .and(users_teams::is_accepted)),
            )
            .select(SHORTCUT_COLUMNS)
            .order_by((shortcut.asc(), users_teams::rank.asc()))
            .get_results(conn)
    }

    pub fn first(
        name: &str,
        user: &User,
        conn: &mut DbConn,
    ) -> Result<Option<Shortcut>, diesel::result::Error> {
        shortcuts
            .inner_join(
                users_teams::table.on(team_slug
                    .eq(users_teams::team_slug)
                    .and(users_teams::user_mail.eq(&user.mail))
                    .and(users_teams::is_accepted)),
            )
            .filter(shortcut.eq(name))
            .select(SHORTCUT_COLUMNS)
            .order_by(users_teams::rank.asc())
            .first::<Shortcut>(conn)
            .optional()
    }

    pub fn upsert(
        new_shortcut: NewShortcut,
        conn: &mut DbConn,
    ) -> Result<Shortcut, diesel::result::Error> {
        diesel::insert_into(shortcuts::table)
            .values(new_shortcut.clone())
            .on_conflict((shortcuts::shortcut, shortcuts::team_slug))
            .do_update()
            .set(UpdatableShortcut { url: new_shortcut.url.clone(), team_slug: new_shortcut.team_slug.clone() })
            .get_result(conn)
    }

    pub fn delete(
        shortcut_name: &str,
        team_slug_name: &str,
        conn: &mut DbConn,
    ) -> Result<(), diesel::result::Error> {
        diesel::delete(shortcuts::table)
            .filter(shortcut.eq(shortcut_name).and(team_slug.eq(team_slug_name)))
            .execute(conn)
            .map(|_| ())
    }
}

#[derive(Insertable, Clone)]
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