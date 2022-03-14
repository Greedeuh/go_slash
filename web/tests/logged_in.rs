use rocket::http::{Cookie, Header, Status};
mod helpers;
use helpers::*;

use go_web::guards::SESSION_COOKIE;
use serde_json::json;

#[test]
fn should_be_logged_in() {
    let client = launch_with(
        "",
        "---
    login:
      simple: true
    ",
        "",
        "some_session_id: some_mail@mail.com",
    );

    assert_eq!(
        client.get("/go/features").dispatch().status(),
        Status::Unauthorized
    );
    assert_eq!(
        client
            .patch("/go/features")
            .json(&json!({ "login": null }))
            .dispatch()
            .status(),
        Status::Unauthorized
    );
}

#[test]
fn should_be_logged_in_is_ok_with_cookie() {
    let client = launch_with(
        "",
        "---
    login:
      simple: true
    ",
        "",
        "some_session_id: some_mail@mail.com",
    );

    assert_ne!(
        client
            .get("/go/features")
            .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .dispatch()
            .status(),
        Status::Unauthorized
    );

    assert_ne!(
        client
            .patch("/go/features")
            .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .dispatch()
            .status(),
        Status::Unauthorized
    );
}

#[test]
fn should_be_logged_in_is_ok_with_header() {
    let client = launch_with(
        "",
        "---
    login:
      simple: true
    ",
        "",
        "some_session_id: some_mail@mail.com",
    );

    assert_ne!(
        client
            .get("/go/features")
            .header(Header::new("Authorization", "some_session_id"))
            .dispatch()
            .status(),
        Status::Unauthorized
    );

    assert_ne!(
        client
            .patch("/go/features")
            .header(Header::new("Authorization", "some_session_id"))
            .dispatch()
            .status(),
        Status::Unauthorized
    );
}
