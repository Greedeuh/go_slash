use rocket::http::Status;
use rocket_dyn_templates::Template;
use serde_json::{json, Value};

use crate::models::AppError;

pub mod features;
pub mod shortcuts;
pub mod users;

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
        }
    }
}

impl From<AppError> for (Status, Template) {
    fn from(e: AppError) -> Self {
        match e {
            AppError::Db => (Status::InternalServerError, Template::render("error", "")),
            AppError::Disable => (Status::Conflict, Template::render("error", "")),
            AppError::Unauthorized => (Status::Unauthorized, Template::render("error", "")),
        }
    }
}
