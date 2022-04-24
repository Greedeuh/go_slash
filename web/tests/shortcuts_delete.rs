use go_web::guards::SESSION_COOKIE;
use go_web::models::features::Features;
use go_web::models::features::LoginFeature;
use go_web::models::users::Capability;
use rocket::http::Cookie;
use rocket::http::Header;
use rocket::http::Status;
mod utils;
use utils::*;

#[test]
fn delete_a_shortcut_return_200() {
    let (client, _conn) = launch_with("");
    let response = client.delete("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn delete_a_shortcut_with_team_return_200() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", false, false, &conn);
    shortcut("myShortCut/hop", "http://localhost", "slug1", &conn);
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

    let shortcut = get_shortcut("myShortCut/hop", &conn);
    assert!(shortcut.is_some());

    let response = client
        .delete("/myShortCut/hop?team=slug1")
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    let shortcut = get_shortcut("myShortCut/hop", &conn);
    assert!(shortcut.is_none());

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn delete_a_shortcut_return_unauthorized() {
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

    let response = client.delete("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn delete_a_shortcut_with_auth_authorized() {
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
