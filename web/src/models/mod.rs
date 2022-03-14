use rustbreak::RustbreakError;
use serde::Deserialize;

pub mod features;
pub mod shortcuts;
pub mod users;

#[derive(Deserialize)]
pub enum AppError {
    Db,
    Disable,
}

impl From<RustbreakError> for AppError {
    fn from(e: RustbreakError) -> Self {
        error!("{:?}", e);
        AppError::Db
    }
}
