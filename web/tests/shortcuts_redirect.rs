use go_web::guards::SESSION_COOKIE;
use go_web::models::features::Features;
use go_web::models::features::LoginFeature;
use rocket::http;
use rocket::http::Cookie;
use rocket::http::Header;
use rocket::http::Status;
mod utils;
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
    shortcut("myShortCut/hop", "https://thetarget.test.go.com", "", &conn);

    let response = client.get("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::PermanentRedirect);
    let mut location = response.headers().get("Location");

    assert_eq!(location.next(), Some("https://thetarget.test.go.com"));
    assert_eq!(location.next(), None);
}

#[test]
fn shortcut_redirect_to_target_based_on_team_rank() {
    let (client, conn) = launch_with(
        "some_session_id: some_mail@mail.com
some_other_session_id: some_other_mail@mail.com",
    );
    team("slug1", "team1", false, true, &conn);
    shortcut("myShortCut/hop", "https://thetarget.test.go.com", "", &conn);
    shortcut(
        "myShortCut/hop",
        "https://theothertarget.test.go.com",
        "slug1",
        &conn,
    );
    user(
        "some_mail@mail.com",
        "pwd",
        &[("", false, 0), ("slug1", false, 1)],
        &[],
        &conn,
    );
    user(
        "some_other_mail@mail.com",
        "pwd",
        &[("", false, 1), ("slug1", false, 0)],
        &[],
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
        .get("/myShortCut/hop")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::PermanentRedirect);
    let mut location = response.headers().get("Location");

    assert_eq!(location.next(), Some("https://thetarget.test.go.com"));
    assert_eq!(location.next(), None);

    let response = client
        .get("/myShortCut/hop")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_other_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::PermanentRedirect);
    let mut location = response.headers().get("Location");

    assert_eq!(location.next(), Some("https://theothertarget.test.go.com"));
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
            ..Default::default()
        },
        &conn,
    );

    let response = client.get("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn shortcut_read_private_should_return_ok_with_session() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    user("some_mail@mail.com", "pwd", &[], &[], &conn);
    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                read_private: true,
                ..Default::default()
            },
            ..Default::default()
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
