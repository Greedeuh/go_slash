use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};

use crate::{
    guards::SessionId,
    models::{
        features::{get_global_features, patch_features, PatchableFeatures},
        users::{should_be_logged_in_if_features, Right, Sessions},
        AppError,
    },
    DbPool,
};

#[get("/go/features")]
pub fn features(
    session_id: Option<SessionId>,
    sessions: &State<Sessions>,
    pool: &State<DbPool>,
) -> Result<Template, (Status, Template)> {
    let conn = pool.get().map_err(AppError::from)?;
    let features = get_global_features(&conn)?;

    should_be_logged_in_if_features(&Right::Admin, &session_id, sessions, &features)?;

    Ok(Template::render(
        "features",
        json!({ "features": json!(features).to_string() }),
    ))
}

#[patch("/go/features", data = "<new_features>")]
pub fn patch_feature(
    new_features: Json<PatchableFeatures>,
    session_id: Option<SessionId>,
    sessions: &State<Sessions>,
    pool: &State<DbPool>,
) -> Result<Status, (Status, Value)> {
    let conn = pool.get().map_err(AppError::from)?;
    let features = get_global_features(&conn)?;

    should_be_logged_in_if_features(&Right::Admin, &session_id, sessions, &features)?;

    patch_features(new_features.into_inner(), &conn)?;

    Ok(Status::Ok)
}
