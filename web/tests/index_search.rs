use diesel::PgConnection;
use go_web::{guards::SESSION_COOKIE};
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
mod utils;
use thirtyfour::prelude::*;
use utils::*;
use thirtyfour_testing_library_ext::{Screen, By as ByExt, TextMatch};

#[async_test]
async fn should_list_shortcuts() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                team("team1", "Team 1", false, true, &mut con);
                shortcut(
                    "newShortcut",
                    &host(port, "/newShortcut"),
                    "",
                    &mut con,
                );
                shortcut(
                    "aShortcut",
                    &host(port, "/aShortcut"),
                    "team1",
                    &mut con,
                );
                shortcut(
                    "ssshortcut",
                    &host(port, "/ssshortcut"),
                    "",
                    &mut con,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("", &[], 0, true), ("team1", &[], 0, true)],
                    &[],
                    &mut con,
                );

                let texts_sorted = [format!("aShortcut {} team1", host(port, "/aShortcut")),
                    format!("newShortcut {}", host(port, "/newShortcut")),
                    format!("ssshortcut {}", host(port, "/ssshortcut"))];
                let href_sorted = [host(port, "/aShortcut"),
                    host(port, "/newShortcut"),
                    host(port, "/ssshortcut")];

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;
                driver.get(host(port, "")).await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                let shortcut_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/Shortcut list/".to_string()))).await?;
                let shortcuts = screen.within(shortcut_list).find_all(ByExt::role("listitem")).await?;

                for i in 0..texts_sorted.len() {

                    assert_eq!(shortcuts[i].text().await?, texts_sorted[i]);
                    assert_eq!(
                        shortcuts[i].attr("href").await?,
                        Some(href_sorted[i].to_owned())
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
async fn should_sugest_when_typing() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                team("slug1", "Team 1", false, true, &mut con);
                shortcut(
                    "newShortcut",
                    &host(port, "/newShortcut"),
                    "",
                    &mut con,
                );
                shortcut(
                    "jeanLuc",
                    &host(port, "/aShortcut"),
                    "slug1",
                    &mut con,
                );
                shortcut(
                    "tadadam",
                    &host(port, "/ssshortcut"),
                    "",
                    &mut con,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("", &[], 0, true), ("slug1", &[], 0, true)],
                    &[],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;
                driver.get(host(port, "")).await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                let shortcut_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/Shortcut list/".to_string()))).await?;
                let scoped_screen = screen.within(shortcut_list);
                let articles = scoped_screen.find_all(ByExt::role("listitem")).await?;
                // initial state
                assert_eq!(3, articles.len());

                let search_bar = screen.get(ByExt::role("searchbox")).await?;
                search_bar.send_keys("t").await?;

                let articles = scoped_screen.find_all(ByExt::role("listitem")).await?;

                // type in t should suggest tadadam first
                assert_eq!(
                    articles[0].text().await?,
                    format!("tadadam {}", host(port, "/ssshortcut"))
                );
                assert_eq!(articles.len(), 3);

                search_bar.send_keys("uc").await?;

                let articles = scoped_screen.find_all(ByExt::role("listitem")).await?;

                // type in tuc should suggest jeanLuc and newShortcut but not tadam
                assert_eq!(
                    articles[0].text().await?,
                    format!("jeanLuc {} slug1", host(port, "/aShortcut"))
                );
                assert_eq!(
                    articles[1].text().await?,
                    format!("newShortcut {}", host(port, "/newShortcut"))
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
async fn should_redirect_when_using_shortcut_with_click() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                shortcut(
                    "newShortcut",
                    &host(port, "/newShortcut"),
                    "",
                    &mut con,
                );
                shortcut(
                    "jeanLuc",
                    &host(port, "/aShortcut1"),
                    "",
                    &mut con,
                );
                shortcut(
                    "tadadam",
                    &host(port, "/ssshortcut"),
                    "",
                    &mut con,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("", &[], 0, true)],
                    &[],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;
                driver.get(host(port, "")).await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                let search_bar = screen.get(ByExt::role("searchbox")).await?;
                search_bar.send_keys("jeanLuc").await?;

                screen
                    .get(ByExt::role("button"))
                    .await?
                    .click()
                    .await?;
                sleep();

                assert_eq!(
                    driver.current_url().await?.to_string(),
                    host(port, "/aShortcut1")
                );

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn should_redirect_when_using_shortcut_with_keyboard() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                shortcut(
                    "newShortcut",
                    &host(port, "/newShortcut"),
                    "",
                    &mut con,
                );
                shortcut(
                    "jeanLuc",
                    &host(port, "/aShortcut1"),
                    "",
                    &mut con,
                );
                shortcut(
                    "tadadam",
                    &host(port, "/ssshortcut"),
                    "",
                    &mut con,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("", &[], 0, true)],
                    &[],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;
                driver.get(host(port, "")).await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                let search_bar = screen.get(ByExt::role("searchbox")).await?;
                search_bar.send_keys(Key::Down).await?;

                let shortcut_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/Shortcut list/".to_string()))).await?;
                let scoped_screen = screen.within(shortcut_list);
                let articles = scoped_screen.find_all(ByExt::role("listitem")).await?;

                // down arrow select first
                assert_eq!(
                    articles[0].text().await?,
                    format!("jeanLuc {}", host(port, "/aShortcut1"))
                );
                assert!(articles[0].class_name().await?.unwrap().contains("active"));

                search_bar.send_keys(Key::Down).await?;

                // down arrow again select snd & unselect first
                let articles = scoped_screen.find_all(ByExt::role("listitem")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("jeanLuc {}", host(port, "/aShortcut1"))
                );
                assert!(!articles[0].class_name().await?.unwrap().contains("active"));
                assert_eq!(
                    articles[1].text().await?,
                    format!("newShortcut {}", host(port, "/newShortcut"))
                );
                assert!(articles[1].class_name().await?.unwrap().contains("active"));

                search_bar.send_keys(Key::Up).await?;

                // up arrow select first & unselect first
                let articles = scoped_screen.find_all(ByExt::role("listitem")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("jeanLuc {}", host(port, "/aShortcut1"))
                );
                assert!(articles[0].class_name().await?.unwrap().contains("active"));
                assert_eq!(
                    articles[1].text().await?,
                    format!("newShortcut {}", host(port, "/newShortcut"))
                );
                assert!(!articles[1].class_name().await?.unwrap().contains("active"));

                search_bar.send_keys(Key::Tab).await?;

                // Tab take first
                assert_eq!(
                    search_bar.prop("value").await?,
                    Some("jeanLuc".to_owned())
                );

                search_bar.send_keys(Key::Enter).await?;
                sleep();

                // Enter launch search
                assert_eq!(
                    driver.current_url().await?.to_string(),
                    host(port, "/aShortcut1")
                );

                driver.get(host(port, "")).await?;
                // arow down then enter go to the first line shortcut

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                let search_bar = screen.get(ByExt::role("searchbox")).await?;
                search_bar.send_keys(Key::Down).await?;
                search_bar.send_keys(Key::Enter).await?;

                sleep();

                assert_eq!(
                    driver.current_url().await?.to_string(),
                    host(port, "/aShortcut1")
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}
