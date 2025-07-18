use diesel::PgConnection;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
mod utils;
use thirtyfour::prelude::*;
use thirtyfour_testing_library_ext::{Screen, By as ByExt};
use utils::*;

#[async_test]
async fn not_logged_in_should_redirect_to_login() {
    in_browser(
        "",
        |driver: &WebDriver, _con: Mutex<PgConnection>, port: u16| {
            async move {
                driver.get(host(port, "")).await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                assert!(screen.find(ByExt::label_text("Mail")).await.is_ok());
                assert!(screen.find(ByExt::label_text("Password")).await.is_ok());
                assert!(screen
                    .find(ByExt::text("Login with google"))
                    .await
                    .is_ok());

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}
