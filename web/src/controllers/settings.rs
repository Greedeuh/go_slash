use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};

use crate::{
    models::{
        settings::{patch_features, Features, PatchableFeatures},
        users::{Capability, User},
        AppError,
    },
    DbPool,
};

#[get("/go/settings")]
pub fn settings(user: User, features: Features) -> Result<Template, (Status, Template)> {
    user.should_have_capability(Capability::Features)?;

    Ok(Template::render(
        "settings",
        json!({ "features_str": json!(features).to_string(), "features": json!(features) }),
    ))
}

#[patch("/go/settings", data = "<new_features>")]
pub fn patch_settings(
    new_features: Json<PatchableFeatures>,
    user: User,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    user.should_have_capability(Capability::Features)?;

    let conn = pool.get().map_err(AppError::from)?;
    patch_features(new_features.into_inner(), &conn)?;

    Ok(Status::Ok)
}
