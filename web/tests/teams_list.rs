use diesel::SqliteConnection;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::http::Status;
use rocket::tokio::sync::Mutex;
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