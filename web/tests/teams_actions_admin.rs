use diesel::SqliteConnection;
use go_web::models::teams::Team;
use rocket::futures::FutureExt;
use rocket::http::Status;
use rocket::tokio::sync::Mutex;
use rocket::{async_test, http};
use serde_json::json;
use thirtyfour::prelude::*;

mod utils;
use go_web::guards::SESSION_COOKIE;
use go_web::models::features::{Features, LoginFeature};
use utils::*;

#[test]
fn delete_team_need_feature() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", false, true, &conn);
    user("some_mail@mail.com", "pwd", true, &[], &conn);
    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                ..Default::default()
            },
            teams: false,
        },
        &conn,
    );

    let response = client
        .delete("/go/teams/slug1")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);

    global_features(
        &Features {
            login: LoginFeature {
                simple: false,
                ..Default::default()
            },
            teams: true,
        },
        &conn,
    );

    let response = client
        .delete("/go/teams/slug1")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);
}

#[test]
fn delete_team_user_need_to_be_admin() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", false, true, &conn);
    user("some_mail@mail.com", "pwd", false, &[], &conn);
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

    let response = client.delete("/go/teams/slug1").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);

    let response = client
        .delete("/go/teams/slug1")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn delete_team() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", false, true, &conn);
    user("some_mail@mail.com", "pwd", true, &[], &conn);
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
        .delete("/go/teams/slug1")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert!(get_team("slug1", &conn).is_none());
}

#[async_test]
async fn admin_action_on_teams() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                team("slug1", "team1", false, false, &con);
                user(
                    "some_mail@mail.com",
                    "pwd",
                    true,
                    &[("slug1", false, 1)],
                    &con,
                );
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            ..Default::default()
                        },
                        teams: true,
                    },
                    &con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                    .await?;

                driver
                    .get(format!("http://localhost:{}/go/teams", port))
                    .await?;

                assert!(driver
                    .find_element(By::Css("button[aria-label='Accept team']"))
                    .await
                    .is_err());

                assert!(driver
                    .find_element(By::Css("button[aria-label='Delete team']"))
                    .await
                    .is_err());

                driver
                    .find_element(By::Css("button[aria-label='Administrate']"))
                    .await?
                    .click()
                    .await?;

                assert!(dbg!(
                    driver
                        .find_element(By::Css("[role='listitem']"))
                        .await?
                        .text()
                        .await?
                )
                .starts_with("team1"));

                let accept_btn = driver
                    .find_element(By::Css("button[aria-label='Accept team']"))
                    .await?;
                accept_btn.click().await?;
                assert!(driver
                    .find_element(By::Css("button[aria-label='Accept team']"))
                    .await
                    .is_err());

                driver
                    .get(format!("http://localhost:{}/go/teams", port))
                    .await?;

                driver
                    .find_element(By::Css("button[aria-label='Administrate']"))
                    .await?
                    .click()
                    .await?;

                assert!(driver
                    .find_element(By::Css("button[aria-label='Accept team']"))
                    .await
                    .is_err());

                let delete_btn = driver
                    .find_element(By::Css("button[aria-label='Delete team']"))
                    .await?;
                delete_btn.click().await?;

                assert!(!dbg!(
                    driver
                        .find_element(By::Css("[role='listitem']"))
                        .await?
                        .text()
                        .await?
                )
                .starts_with("team1"));

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[test]
fn patch_team_need_feature() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", false, true, &conn);
    user("some_mail@mail.com", "pwd", true, &[], &conn);
    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                ..Default::default()
            },
            teams: false,
        },
        &conn,
    );

    let response = client
        .patch("/go/teams/slug1")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .json(&json!({ "title": "newTitle", "is_private": true, "is_accepted": true }))
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);

    global_features(
        &Features {
            login: LoginFeature {
                simple: false,
                ..Default::default()
            },
            teams: true,
        },
        &conn,
    );

    let response = client
        .patch("/go/teams/slug1")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .json(&json!({ "title": "newTitle", "is_private": true, "is_accepted": true }))
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);
}

#[test]
fn patch_team_user_need_to_be_admin() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", false, true, &conn);
    user("some_mail@mail.com", "pwd", false, &[], &conn);
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

    let response = client.delete("/go/teams/slug1").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);

    let response = client
        .patch("/go/teams/slug1")
        .json(&json!({ "title": "newTitle", "is_private": true, "is_accepted": true }))
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn patch_team() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", false, true, &conn);
    user("some_mail@mail.com", "pwd", true, &[], &conn);
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
        .patch("/go/teams/slug1")
        .json(&json!({ "title": "newTitle", "is_private": true, "is_accepted": true }))
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        get_team("slug1", &conn),
        Some(Team {
            slug: "slug1".to_string(),
            title: "newTitle".to_string(),
            is_private: true,
            is_accepted: true
        })
    );

    let response = client
        .patch("/go/teams/slug1")
        .json(&json!({ "title": "newTitle2", }))
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        get_team("slug1", &conn),
        Some(Team {
            slug: "slug1".to_string(),
            title: "newTitle2".to_string(),
            is_private: true,
            is_accepted: true
        })
    );

    let response = client
        .patch("/go/teams/slug1")
        .json(&json!({ "is_private": false }))
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        get_team("slug1", &conn),
        Some(Team {
            slug: "slug1".to_string(),
            title: "newTitle2".to_string(),
            is_private: false,
            is_accepted: true
        })
    );

    let response = client
        .patch("/go/teams/slug1")
        .json(&json!({ "is_accepted": false }))
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        get_team("slug1", &conn),
        Some(Team {
            slug: "slug1".to_string(),
            title: "newTitle2".to_string(),
            is_private: false,
            is_accepted: false
        })
    );
}
