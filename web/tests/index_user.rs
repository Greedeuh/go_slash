use diesel::PgConnection;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
mod utils;
use thirtyfour::prelude::*;
use utils::*;

#[async_test]
async fn not_logged_in_should_redirect_to_login() {
    in_browser(
        "",
        |driver: &WebDriver, _con: Mutex<PgConnection>, port: u16| {
            async move {
                driver.get(host(port, "")).await?;

                assert!(driver.find_element(By::Css("[type='email']")).await.is_ok());
                assert!(driver
                    .find_element(By::Css("[type='password']"))
                    .await
                    .is_ok());
                assert!(driver
                    .find_element(By::Css("[href='/go/login/google']"))
                    .await
                    .is_ok());

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}
