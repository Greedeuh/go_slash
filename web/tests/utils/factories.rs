#[allow(unused_imports)]
pub use no_dead_code::*;

#[allow(dead_code)]
mod no_dead_code {
    use diesel::pg::PgConnection;
    use diesel::prelude::*;
    use go_web::{
        guards::SESSION_COOKIE,
            settings::DEFAULT_CAPABILITIES,
            shortcuts::NewShortcut,
            teams::{Team, TeamCapability},
            users::{Capability, UserTeam, UserWithPwd},
        schema::users,
        schema::{settings, teams},
        schema::{shortcuts, users_teams},
    };
    use serde_json::json;
    use thirtyfour::Cookie;

    pub fn shortcut(shortcut: &str, url: &str, team_slug: &str, db_con: &mut PgConnection) {
        diesel::insert_into(shortcuts::table)
            .values(&NewShortcut {
                shortcut: shortcut.to_string(),
                url: url.to_string(),
                team_slug: team_slug.to_string(),
            })
            .execute(db_con)
            .unwrap();
    }

    pub fn user(
        mail: &str,
        pwd: &str,
        teams: &[(&str, &[TeamCapability], i16, bool)],
        capabilities: &[Capability],
        db_con: &mut PgConnection,
    ) {
        diesel::insert_into(users::table)
            .values(&UserWithPwd {
                mail: mail.to_string(),
                pwd: Some(pwd.to_string()),
                capabilities: capabilities.to_vec(),
            })
            .execute(db_con)
            .unwrap();

        for (team, team_capabilities, rank, is_accepted) in teams {
            diesel::insert_into(users_teams::table)
                .values(&UserTeam {
                    user_mail: mail.to_string(),
                    team_slug: team.to_string(),
                    capabilities: team_capabilities.to_vec(),
                    is_accepted: *is_accepted,
                    rank: *rank,
                })
                .execute(db_con)
                .unwrap();
        }
    }

    pub fn default_capabilities(capabilities: &[Capability], db_con: &mut PgConnection) {
        diesel::update(settings::table)
            .set(settings::content.eq(json!(capabilities).to_string()))
            .filter(settings::title.eq(DEFAULT_CAPABILITIES))
            .execute(db_con)
            .unwrap();
    }

    pub fn team(
        slug: &str,
        title: &str,
        is_private: bool,
        is_accepted: bool,
        db_con: &mut PgConnection,
    ) {
        diesel::insert_into(teams::table)
            .values(&Team {
                slug: slug.to_string(),
                title: title.to_string(),
                is_private,
                is_accepted,
            })
            .execute(db_con)
            .unwrap();
    }

    pub fn session_cookie(session_id: &str, port: u16) -> Cookie {
        let mut cookie = Cookie::new(SESSION_COOKIE, session_id);
        cookie.set_domain(format!("localhost:{port}"));
        cookie
    }

    pub fn session(port: u16) -> Cookie {
        session_cookie("some_session_id", port)
    }
}
