use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::users::Capability;
use crate::models::AppError;
use crate::schema::settings;
use crate::DbConn;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct FeaturesOld {}

pub const DEFAULT_CAPABILITIES: &str = "default_capabilities";
pub const FEATURES: &str = "features";

#[derive(AsChangeset, Queryable, Identifiable, Debug)]
#[table_name = "settings"]
#[primary_key(title)]
pub struct Setting {
    pub title: String,
    pub content: String,
}

pub fn get_global_features(conn: &mut DbConn) -> Result<FeaturesOld, AppError> {
    let features: Setting = settings::table
        .find(FEATURES)
        .first(conn)
        .map_err(AppError::from)?;

    serde_json::from_str(&features.content).map_err(|e| {
        error!("Failed to parse features {}", e);
        AppError::Db
    })
}

pub fn default_capabilities(conn: &mut DbConn) -> Result<Vec<Capability>, AppError> {
    let default_capabilities: Setting = settings::table
        .find(DEFAULT_CAPABILITIES)
        .first(conn)
        .map_err(AppError::from)?;

    serde_json::from_str(&default_capabilities.content).map_err(|e| {
        error!(
            "Can't parse default_capabilities {:?} : {}",
            default_capabilities, e
        );
        AppError::Db
    })
}

pub fn patch_features(_new_features: PatchableFeatures, conn: &mut DbConn) -> Result<usize, AppError> {
    let features = get_global_features(conn)?;

    diesel::update(settings::table)
        .set(settings::content.eq(json!(features).to_string()))
        .filter(settings::title.eq(FEATURES))
        .execute(conn)
        .map_err(AppError::from)
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PatchableFeatures {
    pub login: Option<PatchableLoginFeature>,
    pub teams: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PatchableLoginFeature {
    pub simple: Option<bool>,
    pub google: Option<bool>,
    pub read_private: Option<bool>,
    pub write_private: Option<bool>,
}
