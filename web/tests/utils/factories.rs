pub use no_dead_code::*;

#[allow(dead_code)]
mod no_dead_code {
    use diesel::pg::PgConnection;
    use diesel::prelude::*;
    use go_web::{
        guards::SESSION_COOKIE,
        models::{
            features::Features,
            shortcuts::NewShortcut,
            teams::Team,
            users::{Capability, UserTeam, UserWithPwd},
        },
        schema::global_features,
        schema::teams,
        schema::users,
        schema::{shortcuts, users_teams},
    };
    use serde_json::json;
    use thirtyfour::Cookie;

    pub fn shortcut(shortcut: &str, url: &str, team_slug: &str, db_con: &PgConnection) {
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
        teams: &[(&str, bool, i16)],
        capabilities: &[Capability],
        db_con: &PgConnection,
    ) {
        diesel::insert_into(users::table)
            .values(&UserWithPwd {
                mail: mail.to_string(),
                pwd: pwd.to_string(),
                capabilities: capabilities.to_vec(),
            })
            .execute(db_con)
            .unwrap();

        for (team, is_admin, rank) in teams {
            diesel::insert_into(users_teams::table)
                .values(&UserTeam {
                    user_mail: mail.to_string(),
                    team_slug: team.to_string(),
                    is_admin: *is_admin,
                    is_accepted: true,
                    rank: *rank,
                })
                .execute(db_con)
                .unwrap();
        }
    }

    pub fn global_features(features: &Features, db_con: &PgConnection) {
        diesel::update(global_features::table)
            .set(global_features::features.eq(serde_json::to_string(features).unwrap()))
            .execute(db_con)
            .unwrap();
    }

    pub fn team(
        slug: &str,
        title: &str,
        is_private: bool,
        is_accepted: bool,
        db_con: &PgConnection,
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
        let mut cookie = Cookie::new(SESSION_COOKIE, json!(session_id));
        cookie.set_domain(Some(format!("localhost:{}", port)));
        cookie
    }

    pub fn session(port: u16) -> Cookie {
        session_cookie("some_session_id", port)
    }
}
