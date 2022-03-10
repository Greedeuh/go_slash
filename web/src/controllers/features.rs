use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};

use crate::{models::features::PatchableFeatures, GlobalFeatures};

#[get("/go/features")]
pub fn features(features: &State<GlobalFeatures>) -> Result<Template, (Status, Value)> {
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
) -> Result<Status, (Status, Value)> {
    features.patch(&new_features)?;
    Ok(Status::Ok)
}
