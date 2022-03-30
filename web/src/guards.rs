use rocket::{
    http::{CookieJar, HeaderMap, Status},
    request::{FromRequest, Outcome},
    Request,
};

pub const SESSION_COOKIE: &str = "go_session_id";

#[derive(Clone)]
pub struct SessionId(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SessionId {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(session_id) = session_from_cookies(req.cookies()) {
            return Outcome::Success(session_id);
        }
        if let Some(session_id) = session_from_headers(req.headers()) {
            return Outcome::Success(session_id);
        }

        Outcome::Failure((Status::Unauthorized, ()))
    }
}

fn session_from_cookies(cookies: &CookieJar) -> Option<SessionId> {
    let cookie = match cookies
        .iter()
        .find(|cookie| cookie.name() == SESSION_COOKIE)
    {
        Some(c) => c,
        None => return None,
    };

    Some(SessionId(cookie.value().to_string()))
}

fn session_from_headers(headers: &HeaderMap) -> Option<SessionId> {
    let cookie = match headers
        .iter()
        .find(|header| header.name() == "Authorization")
    {
        Some(c) => c,
        None => return None,
    };

    Some(SessionId(cookie.value().to_string()))
}
