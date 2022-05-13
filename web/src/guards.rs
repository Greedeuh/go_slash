use diesel::prelude::*;
use rocket::{
    http::{CookieJar, HeaderMap},
    outcome::try_outcome,
    request::{FromRequest, Outcome},
    Request, State,
};

use crate::{
    models::{
        users::{Sessions, User, SAFE_USER_COLUMNS},
        AppError,
    },
    schema::users,
    DbPool,
};

pub const SESSION_COOKIE: &str = "go_session_id";

#[derive(Clone)]
pub struct SessionId(pub String);

#[derive(Clone)]
pub struct NonceOIDC(pub String);

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

        Outcome::Failure(AppError::Unauthorized.into())
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for NonceOIDC {
    type Error = serde_json::Value;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let sessions: Outcome<&State<Sessions>, Self::Error> = req
            .guard::<&State<Sessions>>()
            .await
            .map_failure(|_| AppError::Guard.into());
        let sessions = try_outcome!(sessions);

        let session_id = try_outcome!(req.guard::<SessionId>().await);

        match sessions.is_logged_in(&session_id.0) {
            None => {
                error!("Wrong session_id.");
                Outcome::Failure(AppError::Unauthorized.into())
            }
            Some(nonce) => Outcome::Success(NonceOIDC(nonce)),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = serde_json::Value;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let pool: Outcome<&State<DbPool>, Self::Error> = req
            .guard::<&State<DbPool>>()
            .await
            .map_failure(|_| AppError::Guard.into());
        let pool = try_outcome!(pool);

        let sessions: Outcome<&State<Sessions>, Self::Error> = req
            .guard::<&State<Sessions>>()
            .await
            .map_failure(|_| AppError::Guard.into());
        let sessions = try_outcome!(sessions);

        let session_id = try_outcome!(req.guard::<SessionId>().await);

        match get_user(&session_id, sessions, pool) {
            Ok(user) => Outcome::Success(user),
            Err(err) => Outcome::Failure(err.into()),
        }
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

fn get_user(
    session_id: &SessionId,
    sessions: &State<Sessions>,
    pool: &State<DbPool>,
) -> Result<User, AppError> {
    let conn = pool.get().map_err(AppError::from)?;

    match sessions.is_logged_in(&session_id.0) {
        None => {
            error!("Wrong session_id.");
            Err(AppError::Unauthorized)
        }
        Some(mail) => Ok(users::table
            .find(&mail)
            .select(SAFE_USER_COLUMNS)
            .first::<User>(&conn)?),
    }
}
