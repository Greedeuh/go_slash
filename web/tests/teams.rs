use diesel::SqliteConnection;
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
fn feature_team_disable() {
    let (client, _conn) = launch_with("");
    let response = client.get("/go/teams").dispatch();

    assert_eq!(response.status(), Status::Conflict);
}

#[async_test]
async fn layout_with_team_link_if_feature_team() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                user(
                    "some_mail@mail.com",
                    "pwd",
                    true,
                    &[("slug1", false, 0)],
                    &con,
                );

                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            ..Default::default()
                        },
                        teams: false,
                    },
                    &con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                    .await?;

                driver.get(format!("http://localhost:{}", port)).await?;

                assert!(driver
                    .find_element(By::Css("a [href='/go/teams']"))
                    .await
                    .is_err());

                let endpoints = vec!["", "go/teams", "go/features", "azdaz"];

                for endpoint in endpoints {
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
                        .get(format!("http://localhost:{}/{}", port, dbg!(endpoint)))
                        .await?;

                    assert_eq!(
                        driver
                            .find_element(By::Css("[href='/go/teams']"))
                            .await?
                            .text()
                            .await?,
                        "teams"
                    );
                }

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn list_teams_with_infos() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                team("slug1", "team1", false, true, &con);
                team("slug2", "team2", true, true, &con);
                team("slug3", "team3", true, false, &con);
                team("slug4", "team4", false, false, &con);
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

                driver
                    .get(format!("http://localhost:{}/go/teams", port))
                    .await?;

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;

                for i in 0..texts_sorted.len() {
                    let article = &articles[i];
                    print!("{}", i);
                    assert!(article.text().await?.starts_with(texts_sorted[i]));
                    assert_eq!(
                        article.get_attribute("href").await?,
                        Some(href_sorted[i].to_owned())
                    );

                    println!("{}", i);
                    if locks[i] {
                        article.find_element(By::Css(".icon-lock")).await?;
                    } else {
                        assert!(article.find_element(By::Css(".icon-lock")).await.is_err());
                    }
                    if checks[i] {
                        article.find_element(By::Css(".icon-check")).await?;
                        assert!(article
                            .find_element(By::Css(".icon-check-empty"))
                            .await
                            .is_err());
                    } else {
                        assert!(article.find_element(By::Css(".icon-check")).await.is_err());
                        article.find_element(By::Css(".icon-check-empty")).await?;
                    }
                }

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn teams_user_team_then_others() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
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
                    &[("slug1", false, 0)],
                    &con,
                );
                // another user should not change the behaviour
                user(
                    "another@mail.com",
                    "pwd",
                    false,
                    &[("slug2", false, 0), ("slug3", true, 0)],
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

                let user_team = driver
                    .find_element(By::Css("[aria-label='User teams'] [role='listitem']"))
                    .await?;
                assert!(dbg!(user_team.text().await?).starts_with("team1"));

                let other_teams = driver
                    .find_elements(By::Css("[aria-label='Other teams'] [role='listitem']"))
                    .await?;

                let texts_sorted = vec!["Global", "team2", "team3", "team4"];
                for i in 0..texts_sorted.len() {
                    let article = &other_teams[i];
                    assert!(dbg!(article.text().await)?.starts_with(dbg!(texts_sorted[i])));
                }
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[test]
fn post_user_team_need_feature() {
    let (client, _conn) = launch_with("");
    let response = client
        .post("/go/user/teams/slug1")
        .body(json!({ "rank": 0 }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);
}

#[test]
fn post_user_team_need_user() {
    let (client, conn) = launch_with("");

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
        .body(json!({ "rank": 0 }).to_string())
        .dispatch();
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
        .body(json!({ "rank": 0 }).to_string())
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
        &[("slug1", false, 0)],
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
async fn action_on_teams() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                user("some_mail@mail.com", "pwd", false, &[(" ", false, 0)], &con);
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

                let button = driver
                    .find_element(By::Css(
                        "[aria-label='Other teams'] [role='listitem'] button",
                    ))
                    .await?;
                assert_eq!(button.text().await?, "Join");
                button.click().await?;

                assert_eq!(
                    driver
                        .find_element(By::Css(
                            "[aria-label='User teams'] [role='listitem'] button"
                        ))
                        .await?
                        .text()
                        .await?,
                    "Leave"
                );

                driver
                    .get(format!("http://localhost:{}/go/teams", port))
                    .await?;

                let leave = driver
                    .find_element(By::Css(
                        "[aria-label='User teams'] [role='listitem'] button",
                    ))
                    .await?;
                assert_eq!(leave.text().await?, "Leave");
                leave.click().await?;

                assert_eq!(
                    driver
                        .find_element(By::Css(
                            "[aria-label='Other teams'] [role='listitem'] button"
                        ))
                        .await?
                        .text()
                        .await?,
                    "Join"
                );

                driver
                    .get(format!("http://localhost:{}/go/teams", port))
                    .await?;

                assert_eq!(
                    driver
                        .find_element(By::Css(
                            "[aria-label='Other teams'] [role='listitem'] button"
                        ))
                        .await?
                        .text()
                        .await?,
                    "Join"
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[test]
fn put_user_teams_ranks() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", false, true, &conn);
    user(
        "some_mail@mail.com",
        "pwd",
        false,
        &[(" ", false, 0), ("slug1", false, 1)],
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
        .put("/go/user/teams/ranks")
        .body(json!({ " ": 1, "slug1": 0 }).to_string())
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    let users_teams = get_user_team_links("some_mail@mail.com", &conn);

    assert_eq!(
        vec![
            UserTeam {
                user_mail: "some_mail@mail.com".to_string(),
                team_slug: " ".to_string(),
                is_admin: false,
                is_accepted: true,
                rank: 1
            },
            UserTeam {
                user_mail: "some_mail@mail.com".to_string(),
                team_slug: "slug1".to_string(),
                is_admin: false,
                is_accepted: true,
                rank: 0
            }
        ],
        users_teams
    );

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn put_user_teams_ranks_need_feature() {
    let (client, conn) = launch_with("");
    team("slug1", "team1", false, true, &conn);

    let response = client
        .put("/go/user/teams/ranks")
        .body(json!({ " ": 1, "slug1": 0 }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);
}

#[async_test]
async fn user_team_ranks() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                team("slug1", "team1", false, true, &con);
                team("slug2", "team2", false, true, &con);
                team("slug3", "team3", false, true, &con);
                user(
                    "some_mail@mail.com",
                    "pwd",
                    false,
                    &[("", false, 1), ("slug1", false, 2), ("slug2", false, 0)],
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

                assert_eq!(
                    driver
                        .find_element(By::Css("[role='alert']"))
                        .await?
                        .text()
                        .await?,
                    "Drag and drop to prioritize team shortcuts in case of duplicates"
                );
                let teams = driver
                    .find_elements(By::Css("[aria-label='User teams'] [role='listitem'] span"))
                    .await?;

                let expected_teams_sorted = vec!["team2", "Global", "team1"];
                for i in 0..expected_teams_sorted.len() {
                    assert_eq!(expected_teams_sorted[i], teams[i].text().await?);
                }

                driver
                    .find_element(By::Css(
                        "[aria-label='Other teams'] [role='listitem'] button",
                    ))
                    .await?
                    .click()
                    .await?;

                let teams = driver
                    .find_elements(By::Css("[aria-label='User teams'] [role='listitem'] span"))
                    .await?;

                let expected_teams_sorted = vec!["team2", "Global", "team1", "team3"];
                for i in 0..expected_teams_sorted.len() {
                    assert_eq!(expected_teams_sorted[i], teams[i].text().await?);
                }

                // drag & drop testing do not work
                // let teams_rows = driver
                //     .find_elements(By::Css("[aria-label='User teams'] [role='listitem']"))
                //     .await?;

                // driver
                //     .action_chain()
                //     .drag_and_drop_element(
                //         &teams_rows[2],
                //         &driver.find_element(By::Id("draggable")).await?,
                //     )
                //     .perform()
                //     .await?;

                // let teams = driver
                //     .find_elements(By::Css("[aria-label='User teams'] [role='listitem'] span"))
                //     .await?;

                // let expected_teams_sorted = vec!["team1", "team2", "Global"];
                // for i in 0..expected_teams_sorted.len() {
                //     assert_eq!(expected_teams_sorted[i], teams[i].text().await?);
                // }

                // driver
                //     .get(format!("http://localhost:{}/go/teams", port))
                //     .await?;

                // let teams = driver
                //     .find_elements(By::Css("[aria-label='User teams'] [role='listitem'] span"))
                //     .await?;
                // for i in 0..expected_teams_sorted.len() {
                //     assert_eq!(expected_teams_sorted[i], teams[i].text().await?);
                // }

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

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
                team("slug1", "team1", false, true, &con);
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
                    .find_element(By::Css("button[aria-label='Delete team']"))
                    .await
                    .is_err());

                let administrate = driver
                    .find_element(By::Css("button[aria-label='Administrate']"))
                    .await?;
                administrate.click().await?;

                assert!(dbg!(
                    driver
                        .find_element(By::Css("[role='listitem']"))
                        .await?
                        .text()
                        .await?
                )
                .starts_with("team1"));

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
