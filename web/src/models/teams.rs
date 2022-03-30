use diesel::Identifiable;
use serde::Serialize;

use crate::{models::users::UserTeam, schema::teams};

#[derive(Insertable, Queryable, Serialize, Identifiable)]
#[table_name = "teams"]
#[primary_key(slug)]
pub struct Team {
    pub slug: String,
    pub title: String,
    pub is_private: bool,
    pub is_accepted: bool,
}

#[derive(Queryable, Serialize)]
pub struct TeamForUser {
    #[serde(flatten)]
    pub team: Team,
    pub user_link: Option<UserTeam>,
}
