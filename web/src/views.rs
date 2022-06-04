use crate::models::{
    shortcuts::Shortcut,
    teams::{Team, TeamWithUsers},
    users::User,
};

#[derive(Serialize)]
pub struct IndexContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<Shortcut>,
    pub shortcuts: Vec<Shortcut>,
    pub user: User,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<TeamWithUsers>,
    pub teams: Vec<Team>,
}
