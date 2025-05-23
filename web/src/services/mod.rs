use crate::{errors::AppError, services::oidc::OidcServiceError};

pub mod oidc;

impl From<OidcServiceError> for AppError {
    fn from(e: OidcServiceError) -> Self {
        error!("{:?} => AppError::ServiceError", e);
        AppError::ServiceError
    }
}
