use go_web::guards::SESSION_COOKIE;
use go_web::models::features::Features;
use go_web::models::features::LoginFeature;
use go_web::models::shortcuts::Shortcut;
use go_web::models::users::Capability;
use rocket::http::ContentType;
use rocket::http::Cookie;
use rocket::http::Header;
use rocket::http::Status;
mod utils;
use serde_json::json;
use serde_json::Value;
use utils::*;

#[test]
fn create_a_shortcut_with_invalid_url_return_400() {
    let (client, _conn) = launch_with("");

    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .body(r#"{"url": "not_url"}"#)
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_string().unwrap().parse::<Value>().unwrap(),
        json!({"error": "Wrong URL format."})
    );
}

#[test]
fn create_a_shortcut_return_200() {
    let (client, _conn) = launch_with("");
    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .body(r#"{"url": "http://localhost"}"#)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn create_a_shortcut_with_team_return_200() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", false, false, &conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("slug1", true, 0)],
        &[Capability::ShortcutsWrite],
        &conn,
    );
    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                ..Default::default()
            },
            teams: true,
        },
        &conn,
    );

    let response = client
        .put("/myShortCut/hop?team=slug1")
        .header(ContentType::JSON)
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .body(r#"{"url": "http://localhost"}"#)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let shortcut = get_shortcut("myShortCut/hop", &conn);
    assert_eq!(
        shortcut.unwrap(),
        Shortcut {
            shortcut: "myShortCut/hop".to_string(),
            url: "http://localhost".to_string(),
            team_slug: "slug1".to_string()
        }
    );
}

#[test]
fn replace_a_shortcut_return_200() {
    let (client, _conn) = launch_with("");
    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .body(r#"{"url": "http://localhost"}"#)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn put_shortcut_should_return_unauthorized() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                ..Default::default()
            },
            ..Default::default()
        },
        &conn,
    );

    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .body(r#"{"url": "http://localhost"}"#)
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn put_shortcut_should_is_ok_with_auth() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    user(
        "some_mail@mail.com",
        "pwd",
        &[],
        &[Capability::ShortcutsWrite],
        &conn,
    );
    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                ..Default::default()
            },
            ..Default::default()
        },
        &conn,
    );

    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .body(r#"{"url": "http://localhost"}"#)
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_ne!(response.status(), Status::Unauthorized);

    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .body(r#"{"url": "http://localhost"}"#)
        .header(Header::new("Authorization", "some_session_id"))
        .dispatch();

    assert_ne!(response.status(), Status::Unauthorized);
}
