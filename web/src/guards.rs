use diesel::prelude::*;
use rocket::{
    http::{CookieJar, HeaderMap},
    outcome::try_outcome,
    request::{FromRequest, Outcome},
    Request, State,
};

use crate::{
    models::{
        features::{get_global_features, Features},
        users::{Sessions, User},
        AppError,
    },
    schema::users,
    DbPool,
};

pub const SESSION_COOKIE: &str = "go_session_id";

#[derive(Clone)]
pub struct SessionId(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SessionId {
    type Error = serde_json::Value;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let features = try_outcome!(req.guard::<Features>().await);
        if !features.login.simple {
            return Outcome::Failure(AppError::Disable.into());
        }

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
impl<'r> FromRequest<'r> for Features {
    type Error = serde_json::Value;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let pool: Outcome<&State<DbPool>, Self::Error> = req
            .guard::<&State<DbPool>>()
            .await
            .map_failure(|_| AppError::Guard.into());
        let pool = try_outcome!(pool);

        match get_features(pool) {
            Ok(features) => Outcome::Success(features),
            Err(err) => Outcome::Failure(err.into()),
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
            Ok(features) => Outcome::Success(features),
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

fn get_features(pool: &State<DbPool>) -> Result<Features, AppError> {
    let conn = pool.get().map_err(AppError::from)?;
    get_global_features(&conn)
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
        Some(mail) => Ok(users::table.find(&mail).first::<User>(&conn)?),
    }
}
