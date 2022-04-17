use diesel::Identifiable;
use serde::Serialize;

use crate::{models::users::UserTeam, schema::teams};

pub type AllColumns = (
    teams::slug,
    teams::title,
    teams::is_private,
    teams::is_accepted,
);

pub const TEAM_COLUMNS: AllColumns = (
    teams::slug,
    teams::title,
    teams::is_private,
    teams::is_accepted,
);

#[derive(Insertable, Queryable, Serialize, Identifiable, Debug, PartialEq, Eq)]
#[table_name = "teams"]
#[primary_key(slug)]
pub struct Team {
    pub slug: String,
    pub title: String,
    pub is_private: bool,
    pub is_accepted: bool,
}

#[derive(Queryable, Serialize)]
pub struct TeamForOptUser {
    #[serde(flatten)]
    pub team: Team,
    pub user_link: Option<UserTeam>,
}

#[derive(Queryable, Serialize)]
pub struct TeamForUser {
    #[serde(flatten)]
    pub team: Team,
    pub user_link: UserTeam,
}
