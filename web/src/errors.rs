use rustbreak::RustbreakError;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum AppError {
    Db,
    Disable,
    Unauthorized,
    BadRequest,
    Guard,
    NotFound,
    ServiceError,
}

impl From<RustbreakError> for AppError {
    fn from(e: RustbreakError) -> Self {
        error!("{:?}", e);
        AppError::Db
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(e: diesel::result::Error) -> Self {
        error!("{:?}", e);
        AppError::Db
    }
}

impl From<r2d2::Error> for AppError {
    fn from(e: r2d2::Error) -> Self {
        error!("{:?}", e);
        AppError::Db
    }
}
