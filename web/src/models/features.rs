use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::AppError;
use crate::schema::global_features::dsl;
use crate::DbConn;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct Features {
    pub login: LoginFeature,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct LoginFeature {
    pub simple: bool,
    pub read_private: bool,
    pub write_private: bool,
}

pub fn get_global_features(conn: &DbConn) -> Result<Features, AppError> {
    let features = dsl::global_features
        .select(dsl::features)
        .first::<String>(conn)
        .map_err(AppError::from)?;

    serde_json::from_str(&features).map_err(|e| {
        error!("Failed to parse features {}", e);
        AppError::Db
    })
}

pub fn patch_features(new_features: PatchableFeatures, conn: &DbConn) -> Result<usize, AppError> {
    let mut features = get_global_features(conn)?;

    if let Some(login) = &new_features.login {
        if let Some(simple) = login.simple {
            features.login.simple = simple;
        }
        if let Some(read_private) = login.read_private {
            features.login.read_private = read_private;
        }
        if let Some(write_private) = login.write_private {
            features.login.write_private = write_private;
        }
    }
    diesel::update(dsl::global_features)
        .set(dsl::features.eq(json!(features).to_string()))
        .execute(conn)
        .map_err(AppError::from)
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PatchableFeatures {
    pub login: Option<PatchableLoginFeature>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PatchableLoginFeature {
    pub simple: Option<bool>,
    pub read_private: Option<bool>,
    pub write_private: Option<bool>,
}
