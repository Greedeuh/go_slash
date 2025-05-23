use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};

use crate::{
    settings::{patch_features, PatchableFeatures},
    users::{Capability, User},
    errors::AppError,
    DbPool,
};

#[get("/go/settings")]
pub fn get_settings(user: User) -> Result<Template, (Status, Template)> {
    user.should_have_capability(Capability::Features)?;

    Ok(Template::render("settings", json!({})))
}

#[patch("/go/settings", data = "<new_features>")]
pub fn patch_settings(
    new_features: Json<PatchableFeatures>,
    user: User,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    user.should_have_capability(Capability::Features)?;

    let mut conn = pool.get().map_err(AppError::from)?;
    patch_features(new_features.into_inner(), &mut conn)?;

    Ok(Status::Ok)
}
