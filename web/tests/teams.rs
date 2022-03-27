use diesel::SqliteConnection;
use go_web::guards::SESSION_COOKIE;
use go_web::models::features::{Features, LoginFeature};
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::http::Status;
use rocket::tokio::sync::Mutex;
mod utils;
use serde_json::json;
use thirtyfour::prelude::*;
use utils::*;

#[test]
fn feature_team_disable() {
    let (client, _conn) = launch_with("");
    let response = client.get("/go/teams").dispatch();

    assert_eq!(response.status(), Status::Conflict);
}

#[async_test]
async fn list_teams() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>| {
            async move {
                let con = con.lock().await;
                team("slug1", "team1", false, true, &con);
                team("slug2", "team2", true, true, &con);
                team("slug3", "team3", true, false, &con);
                team("slug4", "team4", false, false, &con);
                user("some_mail@mail.com", "pwd", true, &con);

                global_features(
                    &Features {
                        teams: true,
                        ..Default::default()
                    },
                    &con,
                );

                assert_list_all(driver).await;

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
                assert_list_all(driver).await;
            }
            .boxed()
        },
    )
    .await;
}

async fn assert_list_all(driver: &WebDriver) {
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
        assert_eq!(&article.text().await.unwrap(), texts_sorted[i]);
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
