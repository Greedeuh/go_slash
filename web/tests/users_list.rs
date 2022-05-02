use diesel::PgConnection;
use go_web::models::users::Capability;
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
    let response = client.get("/go/users").dispatch();

    assert_eq!(response.status(), Status::Conflict);
}

#[async_test]
async fn layout_with_users_link_if_feature_login() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                user("some_mail@mail.com", "pwd", &[], &Capability::all(), &con);
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: false,
                            ..Default::default()
                        },
                        teams: true,
                    },
                    &con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                    .await?;

                driver.get(format!("http://localhost:{}", port)).await?;

                assert!(driver
                    .find_element(By::Css("a [href='/go/users']"))
                    .await
                    .is_err());

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
                let endpoints = vec!["", "go/teams", "go/features", "azdaz"];

                for endpoint in endpoints {
                    driver
                        .get(format!("http://localhost:{}/{}", port, dbg!(endpoint)))
                        .await?;

                    assert_eq!(
                        driver
                            .find_element(By::Css("[href='/go/users']"))
                            .await?
                            .text()
                            .await?,
                        "users"
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
async fn list_users() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[],
                    &[Capability::UsersAdmin],
                    &con,
                );
                user(
                    "another_mail@mail.com",
                    "pwd",
                    &[],
                    &[Capability::ShortcutsWrite],
                    &con,
                );
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    &con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                    .await?;

                driver
                    .get(format!("http://localhost:{}/go/users", port))
                    .await?;

                let expected_users = vec!["some_mail@mail.com", "another_mail@mail.com"];

                let users = driver.find_elements(By::Css("[role='listitem']")).await?;
                for i in 0..expected_users.len() {
                    assert_eq!(users[i].text().await?, expected_users[i]);
                }

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}
