use rocket::http::Status;
use rocket_dyn_templates::Template;
use rustbreak::RustbreakError;
use serde::Deserialize;
use serde_json::{json, Value};

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


impl From<AppError> for (Status, Value) {
    fn from(e: AppError) -> Self {
        match e {
            AppError::Db => (
                Status::InternalServerError,
                json!({"error": "Probably a database issue :/"}),
            ),
            AppError::Disable => (Status::Conflict, json!({"error": "Feature disable"})),
            AppError::Unauthorized => (
                Status::Unauthorized,
                json!({"error": "Should you really be there ?"}),
            ),
            AppError::BadRequest => (
                Status::BadRequest,
                json!({"error": "Wrong shortcut format."}),
            ),
            AppError::Guard => (
                Status::InternalServerError,
                json!({"error": "Wow that's weird :/"}),
            ),
            AppError::NotFound => (
                Status::NotFound,
                json!({"error": "Can't found what you requested :/"}),
            ),
            AppError::ServiceError => (
                Status::InternalServerError,
                json!({"error": "Wow that's weird :/"}),
            ),
        }
    }
}

impl From<AppError> for (Status, Template) {
    fn from(e: AppError) -> Self {
        match e {
            AppError::Db => (Status::InternalServerError, Template::render("error", "")),
            AppError::Disable => (Status::Conflict, Template::render("error", "")),
            AppError::Unauthorized => {
                (Status::Unauthorized, Template::render("redirect_login", ""))
            }
            AppError::BadRequest => (Status::BadRequest, Template::render("error", "")),
            AppError::Guard => (Status::InternalServerError, Template::render("error", "")),
            AppError::NotFound => (Status::NotFound, Template::render("error", "")),
            AppError::ServiceError => (Status::InternalServerError, Template::render("error", "")),
        }
    }
}