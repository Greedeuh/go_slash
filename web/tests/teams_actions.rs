use diesel::PgConnection;
use go_web::models::teams::Team;
use go_web::models::users::UserTeam;
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
fn delete_team_need_to_be_admin() {
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
fn cant_delete_global_team() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
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

    client
        .delete("/go/teams/")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert!(get_team("", &conn).is_some());
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
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
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
                    .add_cookie(dbg!(Cookie::new(SESSION_COOKIE, json!("some_session_id"))))
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
fn cant_patch_global_team() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
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

    client
        .patch("/go/teams/")
        .json(&json!({ "title": "newTitle", "is_private": true, "is_accepted": true }))
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(
        get_team("", &conn),
        Some(Team {
            slug: "".to_string(),
            title: "Global".to_string(),
            is_private: false,
            is_accepted: true
        })
    );
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

#[test]
fn create_team_need_feature() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
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
        .post("/go/teams")
        .json(&json!({ "slug": "slug1", "title": "newTitle", "is_private": true}))
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
        .post("/go/teams")
        .json(&json!({ "slug": "slug1", "title": "newTitle", "is_private": true }))
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);
}

#[test]
fn cant_create_already_existing() {
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

    client
        .post("/go/teams")
        .json(&json!({ "slug": "slug1", "title": "newTitle", "is_private": true }))
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert!(get_team("", &conn).is_some());
}

#[test]
fn create_team_as_admin() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
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
        .post("/go/teams")
        .json(&json!({ "slug": "slug1", "title": "newTitle", "is_private": true }))
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(
        get_team("slug1", &conn),
        Some(Team {
            slug: "slug1".to_string(),
            title: "newTitle".to_string(),
            is_private: true,
            is_accepted: true
        })
    );
}

#[test]
fn create_team_as_user() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
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

    let response = client
        .post("/go/teams")
        .json(&json!({ "slug": "slug1", "title": "newTitle", "is_private": true }))
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(
        get_team("slug1", &conn),
        Some(Team {
            slug: "slug1".to_string(),
            title: "newTitle".to_string(),
            is_private: true,
            is_accepted: false
        })
    );
}

#[test]
fn create_team_creator_should_be_in_team_as_admin_with_higher_rank() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug", "title", true, false, &conn);
    user(
        "some_mail@mail.com",
        "pwd",
        false,
        &[("slug", false, 0)],
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
        .post("/go/teams")
        .json(&json!({ "slug": "slug1", "title": "newTitle", "is_private": true }))
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(
        get_user_team_links("some_mail@mail.com", &conn),
        vec![
            UserTeam {
                user_mail: "some_mail@mail.com".to_string(),
                team_slug: "slug".to_string(),
                is_admin: false,
                is_accepted: true,
                rank: 0
            },
            UserTeam {
                user_mail: "some_mail@mail.com".to_string(),
                team_slug: "slug1".to_string(),
                is_admin: true,
                is_accepted: true,
                rank: 1
            }
        ]
    );
}

#[async_test]
async fn create_team_from_teams_page() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                user("some_mail@mail.com", "pwd", true, &[], &con);
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

                assert!(
                    !driver
                        .find_element(By::Css("[role='dialog']"))
                        .await?
                        .is_displayed()
                        .await?
                );

                let create_btn = driver
                    .find_element(By::Css("button[aria-label='Start creating team']"))
                    .await?;
                assert_eq!(create_btn.text().await?, "Create");
                create_btn.click().await?;

                let create_dialog = driver.find_element(By::Css("[role='dialog']")).await?;
                create_dialog.wait_until().displayed().await?;
                assert_eq!(
                    create_dialog
                        .find_element(By::Tag("h5"))
                        .await?
                        .text()
                        .await?,
                    "Create team"
                );

                create_dialog
                    .find_element(By::Name("slug"))
                    .await?
                    .send_keys("slug1")
                    .await?;

                create_dialog
                    .find_element(By::Name("title"))
                    .await?
                    .send_keys("slug1")
                    .await?;

                create_dialog
                    .find_element(By::Name("is_private"))
                    .await?
                    .click()
                    .await?;

                create_dialog
                    .find_element(By::Css("button[aria-label='Create team']"))
                    .await?
                    .click()
                    .await?;

                assert!(dbg!(
                    create_dialog
                        .find_element(By::Css("[aria-label='Create team result']"))
                        .await?
                        .text()
                        .await?
                )
                .starts_with("Success ! Your Admins will now have to validate your team."));

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}
