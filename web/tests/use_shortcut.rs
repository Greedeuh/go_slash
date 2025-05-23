use go_web::guards::SESSION_COOKIE;
use rocket::http;
use rocket::http::Status;
mod utils;
use utils::*;

#[test]
fn but_undefined_return_a_404() {
    let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
    user(
        "some_mail@mail.com",
        "pwd",
        &[("", &[], 0, true)],
        &[],
        &mut conn,
    );

    let response = client
        .get("/myShortCut")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn with_user_redirect_to_target() {
    let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
    shortcut("myShortCut/hop", "https://thetarget.test.go.com", "", &mut conn);
    user(
        "some_mail@mail.com",
        "pwd",
        &[("", &[], 0, true)],
        &[],
        &mut conn,
    );

    let response = client
        .get("/myShortCut/hop")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::PermanentRedirect);
    let mut location = response.headers().get("Location");

    assert_eq!(location.next(), Some("https://thetarget.test.go.com"));
    assert_eq!(location.next(), None);
}

#[test]
fn with_team_redirect_to_target_based_on_team_rank() {
    let (client, mut conn) = launch_with(
        "some_session_id: some_mail@mail.com
some_other_session_id: some_other_mail@mail.com",
    );
    team("slug1", "team1", false, true, &mut conn);
    shortcut("myShortCut/hop", "https://thetarget.test.go.com", "", &mut conn);
    shortcut(
        "myShortCut/hop",
        "https://theothertarget.test.go.com",
        "slug1",
        &mut conn,
    );
    user(
        "some_mail@mail.com",
        "pwd",
        &[("", &[], 0, true), ("slug1", &[], 1, true)],
        &[],
        &mut conn,
    );
    user(
        "some_other_mail@mail.com",
        "pwd",
        &[("", &[], 1, true), ("slug1", &[], 0, true)],
        &[],
        &mut conn,
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
fn as_unknown_user_is_not_allowed() {
    let (client, _conn) = launch_with("");

    let response = client.get("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}
