use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use go_web::{
    models::{features::Features, shortcuts::NewShortcut, users::NewUser},
    schema::global_features,
    schema::shortcuts,
    schema::users,
};

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

#[allow(dead_code)]
pub fn user(mail: &str, pwd: &str, db_con: &SqliteConnection) {
    diesel::insert_into(users::table)
        .values(&NewUser {
            mail: mail.to_string(),
            pwd: pwd.to_string(),
        })
        .execute(db_con)
        .unwrap();
}

#[allow(dead_code)]
pub fn global_features(features: &Features, db_con: &SqliteConnection) {
    diesel::update(global_features::table)
        .set(global_features::features.eq(serde_json::to_string(features).unwrap()))
        .execute(db_con)
        .unwrap();
}
