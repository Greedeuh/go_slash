use diesel::dsl::count;
use diesel::prelude::*;
use rocket::{
    http::{Cookie, CookieJar, SameSite, Status},
    response::Redirect,
    serde::json::Json,
    State,
};
use rocket_dyn_templates::Template;
use serde::Deserialize;
use serde_json::{json, Value};
use sha256::digest;
use uuid::Uuid;

use crate::{
    controllers::users::MAIL_REGEX,
    guards::{NonceOIDC, SessionId, SESSION_COOKIE},
    models::{
        settings::default_capabilities,
        users::{Sessions, User, UserWithPwd},
        AppError,
    },
    schema::users,
    services::oidc::OidcService,
    AppConfig, DbConn, DbPool,
};

#[get("/go/login")]
pub fn login(conf: &State<AppConfig>) -> Result<Template, (Status, Template)> {
    Ok(Template::render(
        "login",
        json!({ "context": json!({ "simple_salt": conf.simple_login_salt1 }).to_string() }),
    ))
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
    sessions: &State<Sessions>,
    config: &State<AppConfig>,

    pool: &State<DbPool>,
) -> Result<Json<LoginSuccessfull>, (Status, Value)> {
    let credentials = credentials.into_inner();
    if !MAIL_REGEX.is_match(&credentials.mail) {
        return Err((Status::BadRequest, json!({"error": "Wrong mail format."})));
    }

    let pwd = digest(format!("{}{}", credentials.pwd, config.simple_login_salt2));

    let conn = pool.get().map_err(AppError::from)?;
    let mail_pwd_match: i64 = users::table
        .select(count(users::mail))
        .filter(users::pwd.eq(&pwd))
        .find(&credentials.mail)
        .first(&conn)
        .map_err(AppError::from)?;

    if mail_pwd_match != 1 {
        return Err((
            Status::Unauthorized,
            json!({ "error": "Wrong credentials." }),
        ));
    };

    let token = Uuid::new_v4();
    sessions.put(&token.to_simple().to_string(), &credentials.mail);

    Ok(Json(LoginSuccessfull {
        token: token.to_simple().to_string(),
    }))
}

#[get("/go/login/google")]
pub async fn google_login(
    user: Option<User>,
    sessions: &State<Sessions>,
    oidc_service: &State<OidcService>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, (Status, Template)> {
    if user.is_some() {
        return Err(AppError::BadRequest.into());
    }

    let (auth_url, nonce) = oidc_service.redirect().map_err(AppError::from)?;

    let token = Uuid::new_v4();
    let token = token.to_simple().to_string();
    sessions.put(&token, &nonce);

    cookies.add(
        Cookie::build(SESSION_COOKIE, token)
            .path("/")
            .secure(true)
            .same_site(SameSite::Lax)
            .finish(),
    );

    Ok(Redirect::permanent(auth_url))
}

#[allow(clippy::too_many_arguments)]
#[get("/go/login/redirect/google?<code>")]
pub async fn login_redirect_google(
    code: String,
    nonce: NonceOIDC,
    session_id: SessionId,

    oidc_service: &State<OidcService>,
    sessions: &State<Sessions>,
    pool: &State<DbPool>,
) -> Result<Redirect, (Status, Template)> {
    let token_res = oidc_service
        .retrieve_token(&code, &nonce.0)
        .await
        .map_err(AppError::from)?;

    let mail = token_res.mail;

    let conn = pool.get().map_err(AppError::from)?;
    let matching_user: i64 = users::table
        .select(count(users::mail))
        .find(&mail)
        .first(&conn)
        .map_err(AppError::from)?;

    if matching_user == 0 {
        register_user(&mail, &conn)?;
    }

    sessions.put(&session_id.0, &mail);

    Ok(Redirect::permanent("/".to_string()))
}

fn register_user(mail: &str, conn: &DbConn) -> Result<(), AppError> {
    let capabilities = default_capabilities(conn)?;

    diesel::insert_into(users::table)
        .values(UserWithPwd {
            mail: mail.to_string(),
            capabilities,
            pwd: None,
        })
        .execute(conn)
        .map_err(AppError::from)?;
    Ok(())
}
