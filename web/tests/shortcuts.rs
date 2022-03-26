use go_web::guards::SESSION_COOKIE;
use go_web::models::features::Features;
use go_web::models::features::LoginFeature;
use rocket::http::ContentType;
use rocket::http::Cookie;
use rocket::http::Header;
use rocket::http::Status;
mod utils;
use serde_json::json;
use serde_json::Value;
use utils::*;

#[test]
fn undefined_shortcut_return_a_404() {
    let client = launch_empty();
    let response = client.get("/myShortCut").dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn shortcut_redirect_to_target() {
    let (client, conn) = launch_with("");
    shortcut("myShortCut/hop", "https://thetarget.test.go.com", &conn);

    let response = client.get("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::PermanentRedirect);
    let mut location = response.headers().get("Location");

    assert_eq!(location.next(), Some("https://thetarget.test.go.com"));
    assert_eq!(location.next(), None);
}

#[test]
fn shortcut_read_private_should_return_unauthorized() {
    let (client, conn) = launch_with("");

    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                read_private: true,
                ..Default::default()
            },
        },
        &conn,
    );

    let response = client.get("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn shortcut_read_private_should_return_ok_with_session() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");

    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                read_private: true,
                ..Default::default()
            },
        },
        &conn,
    );

    let response = client
        .get("/myShortCut/hop")
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_ne!(response.status(), Status::Unauthorized);

    let response = client
        .get("/myShortCut/hop")
        .header(Header::new("Authorization", "some_session_id"))
        .dispatch();

    assert_ne!(response.status(), Status::Unauthorized);
}

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
                write_private: true,
                ..Default::default()
            },
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
    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                write_private: true,
                ..Default::default()
            },
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

#[test]
fn delete_a_shortcut_return_200() {
    let (client, _conn) = launch_with("");
    let response = client.delete("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn delete_a_shortcut_return_unauthorized() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                write_private: true,
                ..Default::default()
            },
        },
        &conn,
    );

    let response = client.delete("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn delete_a_shortcut_with_auth_authorized() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                write_private: true,
                ..Default::default()
            },
        },
        &conn,
    );

    let response = client
        .delete("/myShortCut/hop")
        .header(Header::new("Authorization", "some_session_id"))
        .dispatch();
    assert_ne!(response.status(), Status::Unauthorized);

    let response = client
        .delete("/myShortCut/hop")
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();
    assert_ne!(response.status(), Status::Unauthorized);
}
