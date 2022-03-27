use diesel::SqliteConnection;
use go_web::controllers::users::LoginSuccessfull;
use go_web::models::features::{Features, LoginFeature};
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
mod utils;
use rocket::http::Status;
use serde_json::json;
use thirtyfour::prelude::*;
use utils::*;
use uuid::Uuid;

async fn user_can_ask_to_create_a_team() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>| {
            async move {
                let con = con.lock().await;
                shortcut("newShortcut", "http://localhost:8001/newShortcut", "", &con);
                shortcut("aShortcut", "http://localhost:8001/aShortcut", "", &con);
                shortcut("ssshortcut", "http://localhost:8001/ssshortcut", "", &con);

                let texts_sorted = vec![
                    "aShortcut http://localhost:8001/aShortcut",
                    "newShortcut http://localhost:8001/newShortcut",
                    "ssshortcut http://localhost:8001/ssshortcut",
                ];
                let href_sorted = vec![
                    "http://localhost:8001/aShortcut",
                    "http://localhost:8001/newShortcut",
                    "http://localhost:8001/ssshortcut",
                ];

                driver.get("http://localhost:8001").await.unwrap();

                let login_link = driver.find_element(By::Css("a.nav-link")).await.unwrap();
                assert_eq!(
                    login_link.get_attribute("href").await.unwrap(),
                    Some("/go/login".to_owned())
                );
                assert_eq!(login_link.text().await.unwrap(), "Login");

                let articles = driver
                    .find_elements(By::Css("[role='listitem']"))
                    .await
                    .unwrap();

                for i in 0..texts_sorted.len() {
                    assert_eq!(&articles[i].text().await.unwrap(), texts_sorted[i]);
                    assert_eq!(
                        articles[i].get_attribute("href").await.unwrap(),
                        Some(href_sorted[i].to_owned())
                    );
                }
            }
            .boxed()
        },
    )
    .await;
}
