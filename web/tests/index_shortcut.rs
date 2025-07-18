use diesel::PgConnection;
use go_web::guards::SESSION_COOKIE;
use go_web::teams::TeamCapability;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
mod utils;
use thirtyfour::prelude::*;
use thirtyfour_testing_library_ext::{Screen, By as ByExt, TextMatch};
use utils::*;

#[async_test]
async fn with_no_redirect_return_search_and_edit_form_filled() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                team("team", "team", false, true, &mut con);
                shortcut(
                    "newShortcut",
                    &host(port, "/looped"),
                    "team",
                    &mut con,
                );
                shortcut(
                    "newShortcut2",
                    &host(port, "/claude"),
                    "",
                    &mut con,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("team", &[TeamCapability::ShortcutsWrite], 0, true),("", &[], 0, true)],
                    &[],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;
                driver
                    .get(&format!("{}?no_redirect=true", host(port, "/newShortcut")))
                    .await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                let search_bar = screen.get(ByExt::role("searchbox")).await?;
                assert_eq!(
                    search_bar.prop("value").await?,
                    Some("newShortcut".to_owned())
                );

                let shortcut_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/Shortcut list/".to_string()))).await?;
                let scoped_screen = screen.within(shortcut_list);
                let articles = scoped_screen.find_all(ByExt::role("listitem")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("newShortcut {} team", host(port, "/looped"))
                );
                assert_eq!(articles.len(), 2);

                assert_eq!(
                    screen
                        .get(ByExt::placeholder_text("shortcut"))
                        .await?
                        .prop("value")
                        .await?,
                    Some("newShortcut".to_owned())
                );

                assert_eq!(
                    screen
                        .get(ByExt::placeholder_text("https://my-favorite-tool"))
                        .await?
                        .prop("value")
                        .await?,
                    Some(host(port, "/looped"))
                );

                assert_eq!(
                    screen
                        .find(ByExt::role("combobox"))
                        .await?
                        .prop("value")
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
                let mut con = con.lock().await;
                shortcut(
                    "newShortcut1",
                    &host(port, "/looped"),
                    "",
                    &mut con,
                );
                shortcut(
                    "newShortcut2",
                    &host(port, "/claude"),
                    "",
                    &mut con,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("", &[TeamCapability::ShortcutsWrite], 0, true)],
                    &[],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;

                driver
                    .get(host(port, "/newShortcut"))
                    .await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                assert_eq!(
                    screen
                        .get(ByExt::role("alert"))
                        .await?
                        .text()
                        .await?,
                    "Shortcut \"newShortcut\" does not exist yet."
                );

                let search_bar = screen.get(ByExt::role("searchbox")).await?;
                assert_eq!(
                    search_bar.prop("value").await?,
                    Some("newShortcut".to_owned())
                );

                let shortcut_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/Shortcut list/".to_string()))).await?;
                let scoped_screen = screen.within(shortcut_list);
                let articles = scoped_screen.find_all(ByExt::role("listitem")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("newShortcut1 {}", host(port, "/looped"))
                );
                assert_eq!(articles.len(), 2);

                assert_eq!(
                    screen
                        .get(ByExt::placeholder_text("shortcut"))
                        .await?
                        .prop("value")
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
