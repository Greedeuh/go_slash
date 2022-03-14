use lazy_static::lazy_static;
use regex::Regex;
use rocket::{http::Status, serde::json::Json, State};
use rocket_dyn_templates::Template;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sha256::digest;
use uuid::Uuid;

use crate::{
    models::{users::Sessions, AppError},
    AppConfig, GlobalFeatures, SimpleUsers,
};

lazy_static! {
    static ref MAIL_REGEX: Regex =
        Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#,).unwrap();
}

#[get("/go/login")]
pub fn login(
    features: &State<GlobalFeatures>,
    conf: &State<AppConfig>,
) -> Result<Template, (Status, Template)> {
    let mut context: Map<String, Value> = Map::new();
    if !features.get()?.login.simple {
        return Err(AppError::Disable.into());
    } else {
        context.insert(
            "simple_salt".to_string(),
            Value::String(conf.simple_login_salt1.clone()),
        );
    }

    Ok(Template::render("login", json!(context)))
}

#[derive(Deserialize, Serialize)]
pub struct LoginSuccessfull {
    pub token: String,
}

#[derive(Deserialize)]
pub struct Credentials {
    mail: String,
    pwd: String,
}

#[post("/go/login", data = "<credentials>")]
pub fn simple_login(
    credentials: Json<Credentials>,
    features: &State<GlobalFeatures>,
    users: &State<SimpleUsers>,
    sessions: &State<Sessions>,
    config: &State<AppConfig>,
) -> Result<Json<LoginSuccessfull>, (Status, Value)> {
    if !features.get()?.login.simple {
        return Err(AppError::Disable.into());
    }

    let credentials = credentials.into_inner();
    if !MAIL_REGEX.is_match(&credentials.mail) {
        return Err((Status::BadRequest, json!({"error": "Wrong mail format."})));
    }

    let pwd = digest(format!("{}{}", credentials.pwd, config.simple_login_salt2));

    let user = users.get_matching_pwd(&credentials.mail, &pwd)?;

    let _user = match user {
        Some(user) => user,
        None => {
            return Err((
                Status::Unauthorized,
                json!({ "error": "Wrong credentials." }),
            ))
        }
    };

    let token = Uuid::new_v4();
    sessions.put(&token.to_simple().to_string(), &credentials.mail);

    Ok(Json(LoginSuccessfull {
        token: token.to_simple().to_string(),
    }))
}
