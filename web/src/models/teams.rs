use serde::Serialize;

use crate::schema::teams;

#[derive(Insertable, Queryable, Serialize)]
#[table_name = "teams"]
pub struct Team {
    pub slug: String,
    pub title: String,
    pub is_private: bool,
    pub is_accepted: bool,
}
