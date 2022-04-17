use rocket_dyn_templates::Template;
use serde_json::json;

use crate::models::AppError;

pub struct Config;

pub fn template(context: serde_json::Value, app: Config) -> Result<Template, AppError> {
    Ok(Template::render(
        "index",
        json!({
            "global": {
                // "features": json!(features)
            }
        }),
    ))
}
