pub use no_dead_code::*;

#[allow(dead_code)]
mod no_dead_code {
    use diesel::prelude::*;
    use go_web::{models::shortcuts::Shortcut, schema::shortcuts};

    pub fn get_shortcut(shortcut: &str, conn: &SqliteConnection) -> Option<Shortcut> {
        shortcuts::table
            .find(shortcut)
            .first(conn)
            .optional()
            .unwrap()
    }
}
