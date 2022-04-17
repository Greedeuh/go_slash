use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};

use crate::{
    models::{
        features::{patch_features, Features, PatchableFeatures},
        users::{should_be_logged_in_if_features_with, Right, User},
        AppError,
    },
    DbPool,
};

#[get("/go/features")]
pub fn features(user: Option<User>, features: Features) -> Result<Template, (Status, Template)> {
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
    features: Features,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    should_be_logged_in_if_features_with(&Right::Admin, &user, &features)?;

    let conn = pool.get().map_err(AppError::from)?;
    patch_features(new_features.into_inner(), &conn)?;

    Ok(Status::Ok)
}
