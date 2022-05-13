use go_web::guards::SESSION_COOKIE;
use go_web::models::shortcuts::Shortcut;
use go_web::models::teams::TeamCapability;
use rocket::http;
use rocket::http::ContentType;
use rocket::http::Cookie;
use rocket::http::Status;
mod utils;
use serde_json::json;
use serde_json::Value;
use utils::*;

#[test]
fn with_invalid_url_return_400() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    user(
        "some_mail@mail.com",
        "pwd",
        &[("", &[TeamCapability::ShortcutsWrite], 0, true)],
        &[],
        &conn,
    );

    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .body(r#"{"url": "not_url"}"#)
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_string().unwrap().parse::<Value>().unwrap(),
        json!({"error": "Wrong URL format."})
    );
}

#[test]
fn as_user_with_team_capability_is_ok() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    user(
        "some_mail@mail.com",
        "pwd",
        &[("", &[TeamCapability::ShortcutsWrite], 0, true)],
        &[],
        &conn,
    );

    let response = client
        .put("/myShortCut/hop")
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
            team_slug: "".to_string()
        }
    );
}

#[test]
fn with_specific_team_is_ok() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", false, true, &conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("slug1", &[TeamCapability::ShortcutsWrite], 0, true)],
        &[],
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
fn that_already_exist_replace_it() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    shortcut("myShortCut/hop", "http://to_replace", "", &conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("", &[TeamCapability::ShortcutsWrite], 0, true)],
        &[],
        &conn,
    );

    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .body(r#"{"url": "http://localhost"}"#)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        get_shortcut("myShortCut/hop", &conn),
        Some(Shortcut {
            shortcut: "myShortCut/hop".to_string(),
            team_slug: "".to_string(),
            url: "http://localhost".to_string()
        })
    );
}

#[test]
fn as_unknow_user_is_unauthorized() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("wrong_team", "team1", false, false, &conn);
    shortcut("myShortCut/hop", "http://localhost", "", &conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("wrong_team", &[TeamCapability::ShortcutsWrite], 0, true)],
        &[],
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
fn as_user_without_capability_is_unauthorized() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("wrong_team", "team1", false, false, &conn);
    shortcut("myShortCut/hop", "http://localhost", "", &conn);
    user("some_mail@mail.com", "pwd", &[], &[], &conn);

    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .body(r#"{"url": "http://localhost"}"#)
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn as_user_without_team_capability_is_unauthorized() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("wrong_team", "team1", false, false, &conn);
    shortcut("myShortCut/hop", "http://localhost", "", &conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("wrong_team", &[TeamCapability::ShortcutsWrite], 0, true)],
        &[],
        &conn,
    );

    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .body(r#"{"url": "http://localhost"}"#)
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn as_user_with_team_candidature_not_yet_accepted_is_not_allowed() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    user(
        "some_mail@mail.com",
        "pwd",
        &[("", &[TeamCapability::ShortcutsWrite], 0, false)],
        &[],
        &conn,
    );

    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .body(r#"{"url": "http://localhost"}"#)
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn with_team_not_yet_accepted_is_not_allowed() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("team1", "team1", false, false, &conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("team1", &[TeamCapability::ShortcutsWrite], 0, true)],
        &[],
        &conn,
    );

    let response = client
        .put("/myShortCut/hop?team=team1")
        .header(ContentType::JSON)
        .body(r#"{"url": "http://localhost"}"#)
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}
