use crate::models::{settings::Features, shortcuts::Shortcut, teams::Team, users::User};

#[derive(Serialize)]
pub struct IndexContext {
    pub shortcut: Option<Shortcut>,
    pub shortcuts: Vec<Shortcut>,
    pub user: Option<User>,
    pub features: Features,
    pub team: Option<Team>,
    pub teams: Option<Vec<Team>>,
}
