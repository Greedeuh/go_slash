use diesel::PgConnection;
use go_web::models::settings::{Features, LoginFeature};
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
use std::thread;
use std::time::Duration;
mod utils;
use thirtyfour::prelude::*;
use utils::*;

#[async_test]
async fn not_logged_in_should_redirect_to_login() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
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
