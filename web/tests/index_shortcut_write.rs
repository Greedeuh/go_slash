use diesel::PgConnection;
use go_web::guards::SESSION_COOKIE;
use go_web::teams::TeamCapability;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
mod utils;
use thirtyfour::components::SelectElement;
use thirtyfour::prelude::*;
use thirtyfour_testing_library_ext::{Screen, By as ByExt, TextMatch};
use utils::*;

#[async_test]
async fn as_unknow_user_should_not_be_allowed_to_write_shortcuts() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                shortcut(
                    "newShortcut",
                    &host(port, "/newShortcut"),
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

                driver.get(host(port, "")).await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                assert!(screen
                    .query(ByExt::role("button").name(TextMatch::Exact("Switch administration mode".to_string())))
                    .await?
                    .is_none());

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn as_user_without_capability_should_not_be_allowed_to_write_shortcuts() {
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
                user("some_mail@mail.com", "pwd", &[], &[], &mut con);

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;
                driver.get(host(port, "")).await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                assert!(screen
                    .query(ByExt::role("button").name(TextMatch::Exact("Switch administration mode".to_string())))
                    .await?
                    .is_none());

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn as_user_with_team_candidature_not_yet_accepted_should_not_be_allowed_to_write_shortcuts() {
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
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("", &[TeamCapability::ShortcutsWrite], 0, false)],
                    &[],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;
                driver.get(host(port, "")).await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                assert!(screen
                    .query(ByExt::role("button").name(TextMatch::Exact("Switch administration mode".to_string())))
                    .await?
                    .is_none());

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

mod delete_shortcut {
    use super::*;

    #[async_test]
    async fn as_user_with_team_capability_should_be_able_to_delete_shortcuts() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    shortcut(
                        "jeanLuc",
                        &host(port, "/aShortcut1"),
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
                    driver.get(host(port, "")).await?;

                    let screen = Screen::build_with_testing_library(driver.clone()).await?;
                    screen
                        .find(ByExt::role("button").name(TextMatch::Exact("Switch administration mode".to_string())))
                        .await?
                        .click()
                        .await?;

                    screen
                        .find(ByExt::role("button").name(TextMatch::Exact("Delete shortcut".to_string())))
                        .await?
                        .click()
                        .await?;

                    let shortcut_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/Shortcut list/".to_string()))).await?;
                    let scoped_screen = screen.within(shortcut_list);
                    let articles = scoped_screen.query_all(ByExt::role("listitem")).await?;
                    assert_eq!(articles.len(), 0);

                    driver.refresh().await?;
                    let screen = Screen::build_with_testing_library(driver.clone()).await?;
                    let shortcut_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/Shortcut list/".to_string()))).await?;
                    let scoped_screen = screen.within(shortcut_list);
                    let articles = scoped_screen.query_all(ByExt::role("listitem")).await?;
                    assert_eq!(articles.len(), 0);
                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn as_user_with_wrong_team_capabilities_should_not_be_allowed_to_delete_shortcuts() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("slug", "title", true, true, &mut con);
                    shortcut(
                        "first",
                        &host(port, "/aShortcut1"),
                        "slug",
                        &mut con,
                    );
                    shortcut(
                        "second",
                        &host(port, "/aShortcut1"),
                        "",
                        &mut con,
                    );
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[
                            ("", &[TeamCapability::ShortcutsWrite], 0, true),
                            ("slug", &[], 0, true),
                        ],
                        &[],
                        &mut con,
                    );

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                        .await?;
                    driver.get(host(port, "")).await?;

                    let screen = Screen::build_with_testing_library(driver.clone()).await?;
                    screen
                        .find(ByExt::role("button").name(TextMatch::Exact("Switch administration mode".to_string())))
                        .await?
                        .click()
                        .await?;

                    let shortcut_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/Shortcut list/".to_string()))).await?;
                    let scoped_screen = screen.within(shortcut_list);
                    let articles = scoped_screen.query_all(ByExt::role("listitem")).await?;
                    let first = articles.first().unwrap();
                    let first_screen = screen.within(first.clone());

                    assert!(first_screen
                        .query(ByExt::role("button").name(TextMatch::Exact("Delete shortcut".to_string())))
                        .await?
                        .is_none());

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn as_user_without_team_capability_should_not_be_allowed_to_delete_shortcuts() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    shortcut(
                        "jeanLuc",
                        &host(port, "/aShortcut1"),
                        "",
                        &mut con,
                    );
                    user("some_mail@mail.com", "pwd", &[], &[], &mut con);

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                        .await?;
                    driver.get(host(port, "")).await?;

                    assert!(driver
                        .find(By::Css("[aria-label='Switch administration mode']"))
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
    async fn with_a_team_should_not_be_allowed_to_delete_shortcuts() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("team1", "Team 1", false, true, &mut con);
                    shortcut(
                        "jeanLuc",
                        &host(port, "/aShortcut1"),
                        "team1",
                        &mut con,
                    );
                    user("some_mail@mail.com", "pwd", &[], &[], &mut con);

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                        .await?;
                    driver.get(host(port, "")).await?;

                    assert!(driver
                        .find(By::Css("[aria-label='Switch administration mode']"))
                        .await
                        .is_err());

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }
}

mod create_shortcut {
    use super::*;

    #[async_test]
    async fn as_user_with_team_capability_should_be_able_to_create_shortcuts() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
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
                    assert_create_shortcut_ok(driver, "", port).await;

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn with_specific_team_should_be_able_to_create_shortcuts() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("team1", "team1", true, true, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[("team1", &[TeamCapability::ShortcutsWrite], 0, true)],
                        &[],
                        &mut con,
                    );

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                        .await?;
                    assert_create_shortcut_ok(driver, "team1", port).await;

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn as_user_with_team_candidature_not_yet_accepted_should_not_be_allowed_to_create_shortcuts() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("team", "team", true, true, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[
                            ("", &[TeamCapability::ShortcutsWrite], 0, true),
                            ("team", &[TeamCapability::ShortcutsWrite], 0, false),
                        ],
                        &[],
                        &mut con,
                    );

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                        .await?;
                    driver.get(host(port, "")).await?;

                    let screen = Screen::build_with_testing_library(driver.clone()).await?;
                    screen
                        .find(ByExt::role("button").name(TextMatch::Exact("Switch administration mode".to_string())))
                        .await?
                        .click()
                        .await?;

                    let input = screen.find(ByExt::role("combobox")).await?;
                    let select = SelectElement::new(&input).await.unwrap();
                    for opt in select.options().await?.iter() {
                        assert_ne!(opt.text().await?, "team")
                    }

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn with_team_not_yet_accepted_should_not_be_allowed_to_create_shortcuts() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("team", "team", true, false, &mut con);
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
                    driver.get(host(port, "")).await?;

                    let screen = Screen::build_with_testing_library(driver.clone()).await?;
                    screen
                        .find(ByExt::role("button").name(TextMatch::Exact("Switch administration mode".to_string())))
                        .await?
                        .click()
                        .await?;

                    let input = screen.find(ByExt::role("combobox")).await?;
                    let select = SelectElement::new(&input).await.unwrap();
                    for opt in select.options().await?.iter() {
                        assert_ne!(opt.text().await?, "team")
                    }

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn as_user_with_team_capabilities_but_team_not_yet_accepted_should_not_be_allowed_to_create_shortcuts() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("team", "team", true, false, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[
                            ("", &[TeamCapability::ShortcutsWrite], 0, true),
                            ("team", &[TeamCapability::ShortcutsWrite], 0, true),
                        ],
                        &[],
                        &mut con,
                    );

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                        .await?;
                    driver.get(host(port, "")).await?;

                    let screen = Screen::build_with_testing_library(driver.clone()).await?;
                    screen
                        .find(ByExt::role("button").name(TextMatch::Exact("Switch administration mode".to_string())))
                        .await?
                        .click()
                        .await?;

                    let input = screen.find(ByExt::role("combobox")).await?;
                    let select = SelectElement::new(&input).await.unwrap();
                    for opt in select.options().await?.iter() {
                        assert_ne!(opt.text().await?, "team")
                    }

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    async fn assert_create_shortcut_ok(driver: &WebDriver, team: &str, port: u16) {
        let spacer = if team.is_empty() { "" } else { " " };
        driver
            .get(host(port, ""))
            .await
            .unwrap();

        let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
        screen
            .find(ByExt::role("button").name(TextMatch::Exact("Switch administration mode".to_string())))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        screen
            .get(ByExt::placeholder_text("shortcut"))
            .await
            .unwrap()
            .send_keys("jeanLuc")
            .await
            .unwrap();
        screen
            .get(ByExt::placeholder_text("https://my-favorite-tool"))
            .await
            .unwrap()
            .send_keys(host(port, "/aShortcut"))
            .await
            .unwrap();

        let input = screen.find(ByExt::role("combobox")).await.unwrap();
        let select = SelectElement::new(&input).await.unwrap();
        select.select_by_value(team).await.unwrap();

        screen
            .find(ByExt::role("button").name(TextMatch::Exact("Add shortcut".to_string())))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        let shortcut_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/Shortcut list/".to_string()))).await.unwrap();
        let scoped_screen = screen.within(shortcut_list);
        let article = scoped_screen
            .find(ByExt::role("listitem"))
            .await
            .unwrap();
        assert_eq!(
            article.text().await.unwrap(),
            format!("jeanLuc http://host.docker.internal:{}/aShortcut {}NEW", port, team)
        );

        assert_eq!(
            article.prop("href").await.unwrap(),
            Some(host(port, "/jeanLuc?no_redirect"))
        );

        assert_eq!(
            screen
                .get(ByExt::placeholder_text("shortcut"))
                .await
                .unwrap()
                .prop("value")
                .await
                .unwrap(),
            Some("".to_owned())
        );
        assert_eq!(
            screen
                .get(ByExt::placeholder_text("https://my-favorite-tool"))
                .await
                .unwrap()
                .prop("value")
                .await
                .unwrap(),
            Some("".to_owned())
        );

        assert_eq!(
            screen
                .find(ByExt::role("combobox"))
                .await
                .unwrap()
                .prop("value")
                .await
                .unwrap(),
            Some("".to_owned())
        );

        driver.refresh().await.unwrap();

        let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
        let shortcut_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/Shortcut list/".to_string()))).await.unwrap();
        let scoped_screen = screen.within(shortcut_list);
        let article = scoped_screen
            .find(ByExt::role("listitem"))
            .await
            .unwrap();
        assert_eq!(
            article.text().await.unwrap(),
            format!(
                "jeanLuc http://host.docker.internal:{}/aShortcut{}{}",
                port, spacer, team
            )
        );
    }
}
