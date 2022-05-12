use crate::models::{settings::Features, shortcuts::Shortcut, teams::Team, users::User};

#[derive(Serialize)]
pub struct IndexContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<Shortcut>,
    pub shortcuts: Vec<Shortcut>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    pub features: Features,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<Team>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teams: Option<Vec<Team>>,
}
