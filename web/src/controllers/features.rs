use rocket::{form::validate::Contains, http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{models::features::Feature, GlobalFeatures};

#[get("/go/features")]
pub fn features(features: &State<GlobalFeatures>) -> Result<Template, (Status, Value)> {
    let actives = features.all()?;

    let features = vec![Feature::Login];
    let features: Vec<Value> = features
        .iter()
        .map(|feature| json!({"name": feature,"active":actives.contains(feature)}))
        .collect();

    Ok(Template::render(
        "features",
        json!({
            "features": json!(features).to_string()
        }),
    ))
}

#[derive(Deserialize)]
pub struct ReqFeature {
    name: Feature,
    active: bool,
}

#[put("/go/features", data = "<feature>")]
pub fn put_feature(
    features: &State<GlobalFeatures>,
    feature: Json<ReqFeature>,
) -> Result<Status, (Status, Value)> {
    match feature.into_inner() {
        ReqFeature { name, active: true } => features.activate(&name)?,
        ReqFeature {
            name,
            active: false,
        } => features.desactivate(&name)?,
    };
    Ok(Status::Ok)
}
