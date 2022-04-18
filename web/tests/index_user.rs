use diesel::SqliteConnection;
use go_web::guards::SESSION_COOKIE;
use go_web::models::features::{Features, LoginFeature};
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
use serde_json::json;
use std::thread;
use std::time::Duration;
use thirtyfour::components::select::SelectElement;
mod utils;
use thirtyfour::prelude::*;
use utils::*;

#[async_test]
async fn not_logged_in_should_redirect_to_login() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let conn = con.lock().await;
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            read_private: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    &conn,
                );

                driver.get(format!("http://localhost:{}", port)).await?;
                thread::sleep(Duration::from_secs_f32(0.6));
                assert_eq!(
                    driver.current_url().await?,
                    format!("http://localhost:{}/go/login", port)
                );

                driver
                    .get(format!("http://localhost:{}/shortcut", port))
                    .await?;
                thread::sleep(Duration::from_secs_f32(0.6));
                assert_eq!(
                    driver.current_url().await?,
                    format!("http://localhost:{}/go/login?from=/shortcut", port)
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn logged_in_without_write() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let conn = con.lock().await;
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            write_private: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    &conn,
                );

                driver.get(format!("http://localhost:{}", port)).await?;
                assert_eq!(
                    0,
                    driver.find_elements(By::Id("btn-administer")).await?.len()
                );

                driver
                    .get(format!("http://localhost:{}/shortcut", port))
                    .await?;
                assert_eq!(
                    0,
                    driver.find_elements(By::Id("btn-administer")).await?.len()
                );
                assert_eq!(
                    0,
                    driver.find_elements(By::Id("btn-administer")).await?.len()
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}
