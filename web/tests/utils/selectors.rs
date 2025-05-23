pub use no_dead_code::*;

#[allow(dead_code)]
mod no_dead_code {
    use diesel::prelude::*;
    use go_web::{
        models::{
            shortcuts::Shortcut,
            teams::Team,
            users::{User, UserTeam, SAFE_USER_COLUMNS},
        },
        schema::{shortcuts, teams, users, users_teams},
    };

    pub fn get_shortcut(shortcut: &str, conn: &mut PgConnection) -> Option<Shortcut> {
        shortcuts::table
            .filter(shortcuts::shortcut.eq(shortcut))
            .first(conn)
            .optional()
            .unwrap()
    }

    pub fn get_shortcut_with_team(
        shortcut: &str,
        team: &str,
        conn: &mut PgConnection,
    ) -> Option<Shortcut> {
        shortcuts::table
            .find((shortcut, team))
            .first(conn)
            .optional()
            .unwrap()
    }

    pub fn get_user_team_links(mail: &str, conn: &mut PgConnection) -> Vec<UserTeam> {
        users_teams::table
            .filter(users_teams::user_mail.eq(mail))
            .load(conn)
            .unwrap()
    }

    pub fn get_team(slug: &str, conn: &mut PgConnection) -> Option<Team> {
        teams::table.find(slug).first(conn).optional().unwrap()
    }

    pub fn get_user(mail: &str, conn: &mut PgConnection) -> Option<User> {
        users::table
            .find(mail)
            .select(SAFE_USER_COLUMNS)
            .first(conn)
            .optional()
            .unwrap()
    }
}
