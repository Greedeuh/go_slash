use diesel::SqliteConnection;
use rocket::futures::FutureExt;
use rocket::http::Status;
use rocket::tokio::sync::Mutex;
use rocket::{async_test, http};
use serde_json::json;
use serial_test::serial;
use thirtyfour::prelude::*;

mod utils;
use go_web::guards::SESSION_COOKIE;
use go_web::models::features::{Features, LoginFeature};
use utils::*;

#[test]
fn feature_team_disable() {
    let (client, _conn) = launch_with("");
    let response = client.get("/go/teams").dispatch();

    assert_eq!(response.status(), Status::Conflict);
}

#[async_test]
#[serial]
async fn list_teams_with_icons() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>| {
            async move {
                let con = con.lock().await;
                team("slug1", "team1", false, true, &con);
                team("slug2", "team2", true, true, &con);
                team("slug3", "team3", true, false, &con);
                team("slug4", "team4", false, false, &con);
                user(
                    "some_mail@mail.com",
                    "pwd",
                    true,
                    &[
                        ("slug1", false),
                        ("slug2", false),
                        ("slug3", false),
                        ("slug4", false),
                    ],
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
                    .await
                    .unwrap();
                let texts_sorted = vec!["Global", "team1", "team2", "team3", "team4"];
                let href_sorted = vec![
                    "/go/teams/",
                    "/go/teams/slug1",
                    "/go/teams/slug2",
                    "/go/teams/slug3",
                    "/go/teams/slug4",
                ];
                let locks = vec![false, false, true, true, false];
                let checks = vec![true, true, true, false, false];

                driver.get("http://localhost:8001/go/teams").await.unwrap();

                let articles = driver
                    .find_elements(By::Css("[role='listitem']"))
                    .await
                    .unwrap();

                for i in 0..texts_sorted.len() {
                    let article = &articles[i];
                    print!("{}", i);
                    assert!(article.text().await.unwrap().starts_with(texts_sorted[i]));
                    assert_eq!(
                        article.get_attribute("href").await.unwrap(),
                        Some(href_sorted[i].to_owned())
                    );

                    println!("{}", i);
                    if locks[i] {
                        article.find_element(By::Css(".icon-lock")).await.unwrap();
                    } else {
                        assert!(article.find_element(By::Css(".icon-lock")).await.is_err());
                    }
                    if checks[i] {
                        article.find_element(By::Css(".icon-check")).await.unwrap();
                        assert!(article
                            .find_element(By::Css(".icon-check-empty"))
                            .await
                            .is_err());
                    } else {
                        assert!(article.find_element(By::Css(".icon-check")).await.is_err());
                        article
                            .find_element(By::Css(".icon-check-empty"))
                            .await
                            .unwrap();
                    }
                }
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
#[serial]
async fn action_on_teams_list() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>| {
            async move {
                let con = con.lock().await;
                team("slug1", "team1", false, true, &con);
                team("slug2", "team2", true, true, &con);
                team("slug3", "team3", true, false, &con);
                team("slug4", "team4", false, false, &con);
                user(
                    "some_mail@mail.com",
                    "pwd",
                    false,
                    &[("slug1", false)],
                    &con,
                );
                // another user should not change the behaviour
                user(
                    "another@mail.com",
                    "pwd",
                    false,
                    &[("slug2", false), ("slug3", true)],
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

                let texts_sorted = vec!["Global", "team1", "team2", "team3", "team4"];

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                    .await
                    .unwrap();

                driver.get("http://localhost:8001/go/teams").await.unwrap();

                let articles = driver
                    .find_elements(By::Css("[role='listitem']"))
                    .await
                    .unwrap();

                for i in 0..texts_sorted.len() {
                    let article = &articles[i];
                    assert!(article.text().await.unwrap().starts_with(texts_sorted[i]));
                }
            }
            .boxed()
        },
    )
    .await;
}

#[test]
fn post_user_team_need_user() {
    let (client, conn) = launch_with("");
    let response = client.post("/go/user/teams/slug1").dispatch();

    assert_eq!(response.status(), Status::Conflict);

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
    let response = client.post("/go/user/teams/slug1").dispatch();
    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn post_user_team() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", false, false, &conn);
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
        .post("/go/user/teams/slug1")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();
    assert_eq!(response.status(), Status::Created);
}

#[test]
fn delete_user_team_need_user() {
    let (client, conn) = launch_with("");
    let response = client.delete("/go/user/teams/slug1").dispatch();

    assert_eq!(response.status(), Status::Conflict);

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
    let response = client.delete("/go/user/teams/slug1").dispatch();
    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn delete_user_team() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    user(
        "some_mail@mail.com",
        "pwd",
        false,
        &[("slug1", false)],
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
        .delete("/go/user/teams/slug1")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[async_test]
#[serial]
async fn action_on_teams() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>| {
            async move {
                let con = con.lock().await;
                team("slug1", "team1", false, false, &con);
                user(
                    "some_mail@mail.com",
                    "pwd",
                    false,
                    &[("slug1", false)],
                    &con,
                );
                // another user should not change the behaviour
                user(
                    "another@mail.com",
                    "pwd",
                    false,
                    &[("slug2", false), ("slug3", true)],
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
                    .await
                    .unwrap();

                driver.get("http://localhost:8001/go/teams").await.unwrap();

                let article = driver
                    .find_element(By::Css("[role='listitem']"))
                    .await
                    .unwrap();

                let button = article.find_element(By::Css("button")).await.unwrap();
                assert_eq!(button.text().await.unwrap(), "Join");
                button.click().await.unwrap();

                assert_eq!(
                    article
                        .find_element(By::Css("button"))
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap(),
                    "Leave"
                );

                driver.get("http://localhost:8001/go/teams").await.unwrap();

                let leave = driver
                    .find_element(By::Css("[role='listitem'] button"))
                    .await
                    .unwrap();
                assert_eq!(leave.text().await.unwrap(), "Leave");
                leave.click().await.unwrap();

                assert_eq!(
                    driver
                        .find_element(By::Css("[role='listitem'] button"))
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap(),
                    "Join"
                );

                driver.get("http://localhost:8001/go/teams").await.unwrap();

                assert_eq!(
                    driver
                        .find_element(By::Css("[role='listitem'] button"))
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap(),
                    "Join"
                );
            }
            .boxed()
        },
    )
    .await;
}