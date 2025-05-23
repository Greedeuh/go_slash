use go_web::guards::SESSION_COOKIE;
use go_web::models::teams::TeamCapability;
use rocket::http::{Cookie, Status};
mod utils;
use utils::*;

#[test]
fn as_user_with_team_capability_is_ok() {
    let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
    shortcut("myShortCut/hop", "http://localhost", "", &mut conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("", &[TeamCapability::ShortcutsWrite], 0, true)],
        &[],
        &mut conn,
    );

    let shortcut = get_shortcut("myShortCut/hop", &mut conn);
    assert!(shortcut.is_some());

    let response = client
        .delete("/myShortCut/hop")
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    let shortcut = get_shortcut("myShortCut/hop", &mut conn);
    assert!(shortcut.is_none());

    assert_eq!(response.status(), Status::Ok);
    assert!(get_shortcut("/myShortCut/hop", &mut conn).is_none());
}

#[test]
fn with_specific_team_is_ok() {
    let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", false, true, &mut conn);
    shortcut("myShortCut/hop", "http://localhost", "slug1", &mut conn);
    shortcut("myShortCut/hop", "http://localhost", "", &mut conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("slug1", &[TeamCapability::ShortcutsWrite], 0, true)],
        &[],
        &mut conn,
    );

    let shortcut = get_shortcut("myShortCut/hop", &mut conn);
    assert!(shortcut.is_some());

    let response = client
        .delete("/myShortCut/hop?team=slug1")
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert!(get_shortcut_with_team("myShortCut/hop", "slug1", &mut conn).is_none());
    assert!(get_shortcut_with_team("myShortCut/hop", "", &mut conn).is_some());

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn as_unknow_user_is_unauthorized() {
    let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
    shortcut("myShortCut/hop", "http://localhost", "", &mut conn);

    let response = client.delete("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn as_user_without_team_capability_is_unauthorized() {
    let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
    team("wrong_team", "team1", false, true, &mut conn);
    shortcut("myShortCut/hop", "http://localhost", "", &mut conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("wrong_team", &[TeamCapability::ShortcutsWrite], 0, true)],
        &[],
        &mut conn,
    );

    let response = client
        .delete("/myShortCut/hop")
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();
    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn as_user_with_team_candidature_not_yet_accepted_is_not_allowed() {
    let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
    shortcut("myShortCut/hop", "http://localhost", "", &mut conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("", &[TeamCapability::ShortcutsWrite], 0, false)],
        &[],
        &mut conn,
    );

    let response = client
        .delete("/myShortCut/hop")
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();
    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn with_team_not_yet_accepted_is_not_allowed() {
    let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
    team("team1", "team1", false, false, &mut conn);
    shortcut("myShortCut/hop", "http://localhost", "team1", &mut conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("team1", &[TeamCapability::ShortcutsWrite], 0, true)],
        &[],
        &mut conn,
    );

    let response = client
        .delete("/myShortCut/hop?team=team1")
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();
    assert_eq!(response.status(), Status::Unauthorized);
}
