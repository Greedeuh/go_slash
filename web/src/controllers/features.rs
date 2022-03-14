use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};

use crate::{
    guards::SessionId,
    models::{
        features::PatchableFeatures,
        users::{should_be_logged_in, Sessions},
    },
    GlobalFeatures,
};

#[get("/go/features")]
pub fn features(
    session_id: Option<SessionId>,
    sessions: &State<Sessions>,
    features: &State<GlobalFeatures>,
) -> Result<Template, (Status, Template)> {
    should_be_logged_in(session_id, sessions, features)?;

    let features = features.get()?;

    Ok(Template::render(
        "features",
        json!({
            "features": json!(features).to_string()
        }),
    ))
}

#[patch("/go/features", data = "<new_features>")]
pub fn patch_feature(
    features: &State<GlobalFeatures>,
    new_features: Json<PatchableFeatures>,
    session_id: Option<SessionId>,
    sessions: &State<Sessions>,
) -> Result<Status, (Status, Value)> {
    should_be_logged_in(session_id, sessions, features)?;

    features.patch(&new_features)?;
    Ok(Status::Ok)
}
