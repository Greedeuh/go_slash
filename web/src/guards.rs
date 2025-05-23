use rocket::{
    http::{CookieJar, HeaderMap},
    outcome::try_outcome,
    request::{FromRequest, Outcome},
    Request, State,
    fairing::{Fairing, Info, Kind},
    http::{MediaType, Status},
    response::Responder,
    Response,
};
use rocket_dyn_templates::Template;
use serde_json::json;

use crate::{
    errors::{
        AppError,
    },
    users::Sessions,
};

use crate::AppConfig;

pub const SESSION_COOKIE: &str = "go_session_id";

#[derive(Clone)]
pub struct SessionId(pub String);


#[rocket::async_trait]
impl<'r> FromRequest<'r> for SessionId {
    type Error = serde_json::Value;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(session_id) = session_from_cookies(req.cookies()) {
            return Outcome::Success(session_id);
        }
        if let Some(session_id) = session_from_headers(req.headers()) {
            return Outcome::Success(session_id);
        }

        Outcome::Error(AppError::Unauthorized.into())
    }
}

fn session_from_cookies(cookies: &CookieJar) -> Option<SessionId> {
    let cookie = cookies
        .iter()
        .find(|cookie| cookie.name() == SESSION_COOKIE)?;

    Some(SessionId(cookie.value().to_string()))
}

fn session_from_headers(headers: &HeaderMap) -> Option<SessionId> {
    let cookie = headers
        .iter()
        .find(|header| header.name() == "Authorization")?;

    Some(SessionId(cookie.value().to_string()))
}


#[derive(Clone)]
pub struct NonceOIDC(pub String);


#[rocket::async_trait]
impl<'r> FromRequest<'r> for NonceOIDC {
    type Error = serde_json::Value;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let sessions: Outcome<&State<Sessions>, Self::Error> = req
            .guard::<&State<Sessions>>()
            .await
            .map_error(|_| AppError::Guard.into());
        let sessions = try_outcome!(sessions);

        let session_id = try_outcome!(req.guard::<SessionId>().await);

        match sessions.is_logged_in(&session_id.0) {
            None => {
                error!("Wrong session_id.");
                Outcome::Error(AppError::Unauthorized.into())
            }
            Some(nonce) => Outcome::Success(NonceOIDC(nonce)),
        }
    }
}


pub struct UnauthorizedAsLogin;

#[rocket::async_trait]
impl Fairing for UnauthorizedAsLogin {
    // This is a request and response fairing named "GET/POST Counter".
    fn info(&self) -> Info {
        Info {
            name: "UnauthorizedAsLogin",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        if let Some(accept) = request.accept() && !accept.media_types().any(|x| x ==&MediaType::HTML) {
            return;
        }
        if response.status() != Status::Unauthorized {
            return;
        }

        info!("UnauthorizedAsLogin processing");

        let conf = match request.guard::<&State<AppConfig>>().await {
            Outcome::Success(x) => x,
            Outcome::Error(e) => {
                error!(
                    "UnauthorizedAsLogin fail getting State<AppConfig>: Failure {:?}",
                    e
                );
                return;
            }
            Outcome::Forward(e) => {
                error!(
                    "UnauthorizedAsLogin fail getting State<AppConfig>: Forward {:?}",
                    e
                );
                return;
            }
        };

        let template = Template::render(
            "login",
            json!({ "context": json!({ "simple_salt": conf.simple_login_salt1 }).to_string() }),
        );

        match template.respond_to(request) {
            Ok(res) => response.merge(res),
            e => error!(
                "UnauthorizedAsLogin fail getting a response template: {:?}",
                e
            ),
        }
    }
}