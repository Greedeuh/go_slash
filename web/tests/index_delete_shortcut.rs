use diesel::PgConnection;
use go_web::guards::SESSION_COOKIE;
use go_web::models::settings::{Features, LoginFeature};
use go_web::models::teams::TeamCapability;
use go_web::models::users::Capability;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
use serde_json::json;
mod utils;
use thirtyfour::prelude::*;
use utils::*;

#[async_test]
async fn as_unknow_user_is_not_allowed() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/newShortcut", port),
                    "",
                    &con,
                );
                user(
                    "some_mail@mail.com",
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

                driver.get(format!("http://localhost:{}", port)).await?;

                assert!(driver
                    .find_element(By::Css("[aria-label='Switch administration mode']"))
                    .await
                    .is_err());

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn as_user_without_capability_is_not_allowed() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/newShortcut", port),
                    "",
                    &con,
                );
                user(
                    "some_mail@mail.com",
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
                driver.get(format!("http://localhost:{}", port)).await?;

                assert!(driver
                    .find_element(By::Css("[aria-label='Switch administration mode']"))
                    .await
                    .is_err());

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn as_user_with_capability() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/newShortcut", port),
                    "",
                    &con,
                );
                user(
                    "some_mail@mail.com",
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
                        teams: true,
                    },
                    &con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                    .await?;
                driver.get(format!("http://localhost:{}", port)).await?;

                driver
                    .find_element(By::Css("[aria-label='Switch administration mode']"))
                    .await?
                    .click()
                    .await?;

                assert_eq!(
                    driver
                        .find_elements(By::Css("[role='listitem']"))
                        .await?
                        .len(),
                    1
                );

                driver
                    .find_element(By::Css("[aria-label='Delete shortcut']"))
                    .await?
                    .click()
                    .await?;

                assert_eq!(
                    driver
                        .find_elements(By::Css("[role='listitem']"))
                        .await?
                        .len(),
                    0
                );

                driver.refresh().await?;
                assert_eq!(
                    driver
                        .find_elements(By::Css("[role='listitem']"))
                        .await?
                        .len(),
                    0
                );

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn as_user_with_team_capability() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                shortcut(
                    "jeanLuc",
                    &format!("http://localhost:{}/aShortcut1", port),
                    "",
                    &con,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("", &[TeamCapability::ShortcutsWrite], 0)],
                    &[],
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
                driver.get(format!("http://localhost:{}", port)).await?;

                driver
                    .find_element(By::Css("[aria-label='Switch administration mode']"))
                    .await?
                    .click()
                    .await?;

                driver
                    .find_element(By::Css("[aria-label='Delete shortcut']"))
                    .await?
                    .click()
                    .await?;

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(articles.len(), 0);

                driver.refresh().await?;
                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(articles.len(), 0);
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn as_user_without_team_capability() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                shortcut(
                    "jeanLuc",
                    &format!("http://localhost:{}/aShortcut1", port),
                    "",
                    &con,
                );
                user("some_mail@mail.com", "pwd", &[], &[], &con);
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
                driver.get(format!("http://localhost:{}", port)).await?;

                assert!(driver
                    .find_element(By::Css("[aria-label='Switch administration mode']"))
                    .await
                    .is_err());

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn with_a_team() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                team("team1", "Team 1", false, true, &con);
                shortcut(
                    "jeanLuc",
                    &format!("http://localhost:{}/aShortcut1", port),
                    "team1",
                    &con,
                );
                user("some_mail@mail.com", "pwd", &[], &[], &con);
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
                driver.get(format!("http://localhost:{}", port)).await?;

                assert!(driver
                    .find_element(By::Css("[aria-label='Switch administration mode']"))
                    .await
                    .is_err());

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}
