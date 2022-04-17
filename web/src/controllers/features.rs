use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};

use crate::{
    models::{
        features::{get_global_features, patch_features, PatchableFeatures},
        users::{should_be_logged_in_if_features_with, Right, User},
        AppError,
    },
    DbPool,
};

#[get("/go/features")]
pub fn features(user: Option<User>, pool: &State<DbPool>) -> Result<Template, (Status, Template)> {
    let conn = pool.get().map_err(AppError::from)?;
    let features = get_global_features(&conn)?;

    should_be_logged_in_if_features_with(&Right::Admin, &user, &features)?;

    Ok(Template::render(
        "features",
        json!({ "features_str": json!(features).to_string(), "features": json!(features) }),
    ))
}

#[patch("/go/features", data = "<new_features>")]
pub fn patch_feature(
    new_features: Json<PatchableFeatures>,
    user: Option<User>,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    let conn = pool.get().map_err(AppError::from)?;
    let features = get_global_features(&conn)?;

    should_be_logged_in_if_features_with(&Right::Admin, &user, &features)?;

    patch_features(new_features.into_inner(), &conn)?;

    Ok(Status::Ok)
}
