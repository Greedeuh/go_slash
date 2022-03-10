use rustbreak::RustbreakError;
use serde::Deserialize;

pub mod features;
pub mod shortcuts;

#[derive(Deserialize)]
pub enum AppError {
    Db,
}

impl From<RustbreakError> for AppError {
    fn from(e: RustbreakError) -> Self {
        error!("{:?}", e);
        AppError::Db
    }
}
