use diesel::SqliteConnection;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
mod utils;
use thirtyfour::prelude::*;
use utils::*;

#[async_test]
async fn shortcut_no_redirect_return_search_filled_and_edit_form() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                shortcut(
                    "newShortcut",
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

                // create shortcut
                driver
                    .get(format!(
                        "http://localhost:{}/newShortcut?no_redirect=true",
                        port
                    ))
                    .await?;

                let login_link = driver.find_element(By::Css("a.nav-link")).await?;
                assert_eq!(
                    login_link.get_attribute("href").await?,
                    Some("/go/login".to_owned())
                );
                assert_eq!(login_link.text().await?, "Login");

                let search_bar = driver.find_element(By::Css("input[type='search']")).await?;
                assert_eq!(
                    search_bar.get_property("value").await?,
                    Some("newShortcut".to_owned())
                );

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("newShortcut http://localhost:{}/looped", port)
                );
                assert_eq!(articles.len(), 2);

                let shortcut_edit = driver
                    .find_element(By::Css("input[name='shortcut']"))
                    .await?;
                assert_eq!(
                    shortcut_edit.get_property("value").await?,
                    Some("newShortcut".to_owned())
                );
                assert_eq!(
                    shortcut_edit.get_property("disabled").await?,
                    Some("true".to_owned())
                );

                driver
                    .find_element(By::Css("input[name='url']"))
                    .await?
                    .send_keys("2")
                    .await?;
                driver
                    .find_element(By::Id("btn-add"))
                    .await?
                    .click()
                    .await?;

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("newShortcut http://localhost:{}/looped2 NEW", port)
                );
                assert_eq!(articles.len(), 2);
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn undefined_shortcut_return_search_filled_and_edit_form() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
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

                // create shortcut
                driver
                    .get(format!("http://localhost:{}/newShortcut", port))
                    .await?;

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

                let shortcut_edit = driver
                    .find_element(By::Css("input[name='shortcut']"))
                    .await?;
                assert_eq!(
                    shortcut_edit.get_property("value").await?,
                    Some("newShortcut".to_owned())
                );
                assert_eq!(
                    shortcut_edit.get_property("disabled").await?,
                    Some("true".to_owned())
                );

                driver
                    .find_element(By::Css("input[name='url']"))
                    .await?
                    .send_keys(format!("http://localhost:{}/ring", port))
                    .await?;
                driver
                    .find_element(By::Id("btn-add"))
                    .await?
                    .click()
                    .await?;

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("newShortcut http://localhost:{}/ring NEW", port)
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}
