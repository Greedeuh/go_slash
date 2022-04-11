use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use go_web::{
    models::{
        features::Features,
        shortcuts::NewShortcut,
        teams::Team,
        users::{User, UserTeam},
    },
    schema::global_features,
    schema::teams,
    schema::users,
    schema::{shortcuts, users_teams},
};

#[allow(dead_code)]
pub fn shortcut(shortcut: &str, url: &str, team_slug: &str, db_con: &SqliteConnection) {
    diesel::insert_into(shortcuts::table)
        .values(&NewShortcut {
            shortcut: shortcut.to_string(),
            url: url.to_string(),
            team_slug: team_slug.to_string(),
        })
        .execute(db_con)
        .unwrap();
}

#[allow(dead_code)]
pub fn user(
    mail: &str,
    pwd: &str,
    admin: bool,
    teams: &[(&str, bool, i16)],
    db_con: &SqliteConnection,
) {
    diesel::insert_into(users::table)
        .values(&User {
            mail: mail.to_string(),
            pwd: pwd.to_string(),
            is_admin: admin,
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

#[allow(dead_code)]
pub fn global_features(features: &Features, db_con: &SqliteConnection) {
    diesel::update(global_features::table)
        .set(global_features::features.eq(serde_json::to_string(features).unwrap()))
        .execute(db_con)
        .unwrap();
}

#[allow(dead_code)]
pub fn team(
    slug: &str,
    title: &str,
    is_private: bool,
    is_accepted: bool,
    db_con: &SqliteConnection,
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
