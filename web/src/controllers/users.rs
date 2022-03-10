use rocket::{http::Status, State};
use rocket_dyn_templates::Template;
use serde_json::{json, Value};

use crate::GlobalFeatures;

#[get("/go/login")]
pub fn login(features: &State<GlobalFeatures>) -> Result<Template, (Status, Value)> {
    if features.get()?.login.simple {}

    Ok(Template::render(
        "login",
        json!({
            "github": "",
            "google": "",
            "simple":""
        }),
    ))
}
