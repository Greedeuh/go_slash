use rocket_dyn_templates::Template;
use serde_json::json;

#[get("/go/login")]
pub fn login() -> Template {
    Template::render(
        "login",
        json!({
            "github": "",
            "google": ""
        }),
    )
}
