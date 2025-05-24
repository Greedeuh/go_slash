use crate::{
    shortcuts::Shortcut,teams::{Team, TeamWithUserLinks},users::User};
use serde::Serialize;

#[derive(Serialize)]
pub struct IndexContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<Shortcut>,
    pub shortcuts: Vec<Shortcut>,
    pub user: User,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<TeamWithUserLinks>,
    pub teams: Vec<Team>,
}
