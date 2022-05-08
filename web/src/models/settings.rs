use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::users::Capability;
use crate::models::AppError;
use crate::schema::{global_features::dsl, settings};
use crate::DbConn;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct Features {
    pub login: LoginFeature,
    pub teams: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct LoginFeature {
    pub simple: bool,
    pub google: bool,
    pub read_private: bool,
}

pub const DEFAULT_CAPABILITIES: &str = "default_capabilities";

#[derive(AsChangeset, Queryable, Identifiable, Debug)]
#[table_name = "settings"]
#[primary_key(title)]
pub struct Setting {
    pub title: String,
    pub content: String,
}

impl LoginFeature {
    pub fn any(&self) -> bool {
        self.simple || self.google
    }
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

pub fn default_capabilities(conn: &DbConn) -> Result<Vec<Capability>, AppError> {
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

pub fn patch_features(new_features: PatchableFeatures, conn: &DbConn) -> Result<usize, AppError> {
    let mut features = get_global_features(conn)?;

    if let Some(login) = &new_features.login {
        if let Some(simple) = login.simple {
            features.login.simple = simple;
        }
        if let Some(google) = login.google {
            features.login.google = google;
        }
        if let Some(read_private) = login.read_private {
            features.login.read_private = read_private;
        }
    }

    if let Some(teams) = new_features.teams {
        features.teams = teams;
    }

    diesel::update(dsl::global_features)
        .set(dsl::features.eq(json!(features).to_string()))
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
