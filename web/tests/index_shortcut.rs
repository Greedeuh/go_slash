use diesel::PgConnection;
use go_web::guards::SESSION_COOKIE;
use go_web::models::teams::TeamCapability;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
mod utils;
use serde_json::json;
use thirtyfour::prelude::*;
use utils::*;

#[async_test]
async fn with_no_redirect_return_search_and_edit_form_filled() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                team("team", "team", false, true, &con);
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/looped", port),
                    "team",
                    &con,
                );
                shortcut(
                    "newShortcut2",
                    &format!("http://localhost:{}/claude", port),
                    "",
                    &con,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("team", &[TeamCapability::ShortcutsWrite], 0, true)],
                    &[],
                    &con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                    .await?;
                driver
                    .get(format!(
                        "http://localhost:{}/newShortcut?no_redirect=true",
                        port
                    ))
                    .await?;

                let search_bar = driver.find_element(By::Css("input[type='search']")).await?;
                assert_eq!(
                    search_bar.get_property("value").await?,
                    Some("newShortcut".to_owned())
                );

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("newShortcut http://localhost:{}/looped team", port)
                );
                assert_eq!(articles.len(), 2);

                assert_eq!(
                    driver
                        .find_element(By::Css("input[name='shortcut']"))
                        .await?
                        .get_property("value")
                        .await?,
                    Some("newShortcut".to_owned())
                );

                assert_eq!(
                    driver
                        .find_element(By::Css("input[name='url']"))
                        .await?
                        .get_property("value")
                        .await?,
                    Some(format!("http://localhost:{}/looped", port))
                );

                assert_eq!(
                    driver
                        .find_element(By::Css("[name='team']"))
                        .await?
                        .get_property("value")
                        .await?
                        .unwrap(),
                    "team"
                );

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn with_not_existing_shortcut_return_search_and_edit_form_filled() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                shortcut(
                    "newShortcut1",
                    &format!("http://localhost:{}/looped", port),
                    "",
                    &con,
                );
                shortcut(
                    "newShortcut2",
                    &format!("http://localhost:{}/claude", port),
                    "",
                    &con,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("", &[TeamCapability::ShortcutsWrite], 0, true)],
                    &[],
                    &con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                    .await?;

                driver
                    .get(format!("http://localhost:{}/newShortcut", port))
                    .await?;

                assert_eq!(
                    driver
                        .find_element(By::Css("[role='alert']"))
                        .await?
                        .text()
                        .await?,
                    "Shortcut \"newShortcut\" does not exist yet."
                );

                let search_bar = driver.find_element(By::Css("input[type='search']")).await?;
                assert_eq!(
                    search_bar.get_property("value").await?,
                    Some("newShortcut".to_owned())
                );

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("newShortcut1 http://localhost:{}/looped", port)
                );
                assert_eq!(articles.len(), 2);

                assert_eq!(
                    driver
                        .find_element(By::Css("input[name='shortcut']"))
                        .await?
                        .get_property("value")
                        .await?,
                    Some("newShortcut".to_owned())
                );

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}
