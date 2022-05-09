use go_web::guards::SESSION_COOKIE;
use go_web::models::settings::Features;
use go_web::models::settings::LoginFeature;
use go_web::models::teams::TeamCapability;
use go_web::models::users::Capability;
use rocket::http::{Cookie, Status};
mod utils;
use utils::*;

#[test]
fn as_user_with_capability_is_ok() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    shortcut("myShortCut/hop", "http://localhost", "", &conn);
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
            teams: true,
        },
        &conn,
    );

    let shortcut = get_shortcut("myShortCut/hop", &conn);
    assert!(shortcut.is_some());

    let response = client
        .delete("/myShortCut/hop")
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    let shortcut = get_shortcut("myShortCut/hop", &conn);
    assert!(shortcut.is_none());

    assert_eq!(response.status(), Status::Ok);

    assert!(get_shortcut("/myShortCut/hop", &conn).is_none());
}

#[test]
fn as_user_with_team_capability_is_ok() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    shortcut("myShortCut/hop", "http://localhost", "", &conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("", &[TeamCapability::ShortcutsWrite], 0)],
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

    let shortcut = get_shortcut("myShortCut/hop", &conn);
    assert!(shortcut.is_some());

    let response = client
        .delete("/myShortCut/hop")
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    let shortcut = get_shortcut("myShortCut/hop", &conn);
    assert!(shortcut.is_none());

    assert_eq!(response.status(), Status::Ok);
    assert!(get_shortcut("/myShortCut/hop", &conn).is_none());
}

#[test]
fn with_specific_team_is_ok() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", false, false, &conn);
    shortcut("myShortCut/hop", "http://localhost", "slug1", &conn);
    shortcut("myShortCut/hop", "http://localhost", "", &conn);
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

    assert!(get_shortcut_with_team("myShortCut/hop", "slug1", &conn).is_none());
    assert!(get_shortcut_with_team("myShortCut/hop", "", &conn).is_some());

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn as_unknow_user_is_unauthorized() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    shortcut("myShortCut/hop", "http://localhost", "", &conn);
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
fn as_user_without_capability_is_unauthorized() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    shortcut("myShortCut/hop", "http://localhost", "", &conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("", &[TeamCapability::ShortcutsWrite], 0)],
        &[],
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
        &[("wrong_team", &[TeamCapability::ShortcutsWrite], 0)],
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
        .delete("/myShortCut/hop")
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();
    assert_eq!(response.status(), Status::Unauthorized);
}
