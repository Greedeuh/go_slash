use diesel::PgConnection;
use go_web::guards::SESSION_COOKIE;
use go_web::models::teams::TeamCapability;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
mod utils;
use thirtyfour::components::SelectElement;
use thirtyfour::prelude::*;
use utils::*;

#[async_test]
async fn as_unknow_user_is_not_allowed() {
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
async fn as_user_without_capability_is_not_allowed() {
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
async fn as_user_with_team_candidature_not_yet_accepted_is_not_allowed() {
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

mod delete_shortcut {
    use super::*;

    #[async_test]
    async fn as_user_with_team_capability() {
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

                    driver
                        .find(By::Css("[aria-label='Switch administration mode']"))
                        .await?
                        .click()
                        .await?;

                    driver
                        .find(By::Css("[aria-label='Delete shortcut']"))
                        .await?
                        .click()
                        .await?;

                    let articles = driver.find_all(By::Css("[role='listitem']")).await?;
                    assert_eq!(articles.len(), 0);

                    driver.refresh().await?;
                    let articles = driver.find_all(By::Css("[role='listitem']")).await?;
                    assert_eq!(articles.len(), 0);
                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn as_user_with_wrong_team_capabilities_is_not_allowed() {
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

                    driver
                        .find(By::Css("[aria-label='Switch administration mode']"))
                        .await?
                        .click()
                        .await?;

                    let articles = driver.find_all(By::Css("[role='listitem']")).await?;
                    let first = articles.first().unwrap();

                    assert!(first
                        .find(By::Css("[aria-label='Delete shortcut']"))
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
    async fn as_user_without_team_capability_is_not_allowed() {
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
    async fn with_a_team() {
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
    async fn as_user_with_team_capability() {
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
    async fn with_specific_team() {
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
    async fn as_user_with_team_candidature_not_yet_accepted_is_not_allowed() {
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

                    driver
                        .find(By::Css("[aria-label='Switch administration mode']"))
                        .await?
                        .click()
                        .await?;

                    let input = driver.find(By::Css("[name='team']")).await.unwrap();
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
    async fn with_team_not_yet_accepted_is_not_allowed() {
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

                    driver
                        .find(By::Css("[aria-label='Switch administration mode']"))
                        .await?
                        .click()
                        .await?;

                    let input = driver.find(By::Css("[name='team']")).await.unwrap();
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
    async fn as_user_with_team_capabilities_but_team_not_yet_accepted_is_not_allowed() {
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

                    driver
                        .find(By::Css("[aria-label='Switch administration mode']"))
                        .await?
                        .click()
                        .await?;

                    let input = driver.find(By::Css("[name='team']")).await.unwrap();
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

        driver
            .find(By::Css("[aria-label='Switch administration mode']"))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        driver
            .find(By::Css("[name='shortcut']"))
            .await
            .unwrap()
            .send_keys("jeanLuc")
            .await
            .unwrap();
        driver
            .find(By::Css("[name='url']"))
            .await
            .unwrap()
            .send_keys(host(port, "/aShortcut"))
            .await
            .unwrap();

        let input = driver.find(By::Css("[name='team']")).await.unwrap();
        let select = SelectElement::new(&input).await.unwrap();
        select.select_by_value(team).await.unwrap();

        driver
            .find(By::Css("[aria-label='Add shortcut']"))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        let article = driver
            .find(By::Css("[role='listitem']"))
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
            driver
                .find(By::Css("[name='shortcut']"))
                .await
                .unwrap()
                .prop("value")
                .await
                .unwrap(),
            Some("".to_owned())
        );
        assert_eq!(
            driver
                .find(By::Css("[name='url']"))
                .await
                .unwrap()
                .prop("value")
                .await
                .unwrap(),
            Some("".to_owned())
        );

        assert_eq!(
            driver
                .find(By::Css("[name='team']"))
                .await
                .unwrap()
                .prop("value")
                .await
                .unwrap(),
            Some("".to_owned())
        );

        driver.refresh().await.unwrap();

        let article = driver
            .find(By::Css("[role='listitem']"))
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
