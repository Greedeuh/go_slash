use diesel::PgConnection;
use go_web::guards::SESSION_COOKIE;
use go_web::models::settings::{Features, LoginFeature};
use go_web::models::teams::TeamCapability;
use go_web::models::users::Capability;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
use serde_json::json;
mod utils;
use thirtyfour::prelude::*;
use utils::*;

#[async_test]
async fn as_unknow_user_is_not_allowed() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/newShortcut", port),
                    "",
                    &con,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[],
                    &[Capability::ShortcutsWrite],
                    &con,
                );
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    &con,
                );

                driver.get(format!("http://localhost:{}", port)).await?;

                assert!(driver
                    .find_element(By::Css("[aria-label='Switch administration mode']"))
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
                let con = con.lock().await;
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/newShortcut", port),
                    "",
                    &con,
                );
                user("some_mail@mail.com", "pwd", &[], &[], &con);
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            ..Default::default()
                        },
                        teams: true,
                    },
                    &con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                    .await?;
                driver.get(format!("http://localhost:{}", port)).await?;

                assert!(driver
                    .find_element(By::Css("[aria-label='Switch administration mode']"))
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
    async fn as_user_with_capability() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let con = con.lock().await;
                    shortcut(
                        "newShortcut",
                        &format!("http://localhost:{}/newShortcut", port),
                        "",
                        &con,
                    );
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[],
                        &[Capability::ShortcutsWrite],
                        &con,
                    );
                    global_features(
                        &Features {
                            login: LoginFeature {
                                simple: true,
                                ..Default::default()
                            },
                            teams: true,
                        },
                        &con,
                    );

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                        .await?;
                    driver.get(format!("http://localhost:{}", port)).await?;

                    driver
                        .find_element(By::Css("[aria-label='Switch administration mode']"))
                        .await?
                        .click()
                        .await?;

                    assert_eq!(
                        driver
                            .find_elements(By::Css("[role='listitem']"))
                            .await?
                            .len(),
                        1
                    );

                    driver
                        .find_element(By::Css("[aria-label='Delete shortcut']"))
                        .await?
                        .click()
                        .await?;

                    assert_eq!(
                        driver
                            .find_elements(By::Css("[role='listitem']"))
                            .await?
                            .len(),
                        0
                    );

                    driver.refresh().await?;
                    assert_eq!(
                        driver
                            .find_elements(By::Css("[role='listitem']"))
                            .await?
                            .len(),
                        0
                    );

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn as_user_with_team_capability() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let con = con.lock().await;
                    shortcut(
                        "jeanLuc",
                        &format!("http://localhost:{}/aShortcut1", port),
                        "",
                        &con,
                    );
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[("", &[TeamCapability::ShortcutsWrite], 0)],
                        &[],
                        &con,
                    );
                    global_features(
                        &Features {
                            login: LoginFeature {
                                simple: true,
                                ..Default::default()
                            },
                            teams: true,
                        },
                        &con,
                    );

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                        .await?;
                    driver.get(format!("http://localhost:{}", port)).await?;

                    driver
                        .find_element(By::Css("[aria-label='Switch administration mode']"))
                        .await?
                        .click()
                        .await?;

                    driver
                        .find_element(By::Css("[aria-label='Delete shortcut']"))
                        .await?
                        .click()
                        .await?;

                    let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                    assert_eq!(articles.len(), 0);

                    driver.refresh().await?;
                    let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                    assert_eq!(articles.len(), 0);
                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn as_user_without_team_capability() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let con = con.lock().await;
                    shortcut(
                        "jeanLuc",
                        &format!("http://localhost:{}/aShortcut1", port),
                        "",
                        &con,
                    );
                    user("some_mail@mail.com", "pwd", &[], &[], &con);
                    global_features(
                        &Features {
                            login: LoginFeature {
                                simple: true,
                                ..Default::default()
                            },
                            teams: true,
                        },
                        &con,
                    );
                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                        .await?;
                    driver.get(format!("http://localhost:{}", port)).await?;

                    assert!(driver
                        .find_element(By::Css("[aria-label='Switch administration mode']"))
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
                    let con = con.lock().await;
                    team("team1", "Team 1", false, true, &con);
                    shortcut(
                        "jeanLuc",
                        &format!("http://localhost:{}/aShortcut1", port),
                        "team1",
                        &con,
                    );
                    user("some_mail@mail.com", "pwd", &[], &[], &con);
                    global_features(
                        &Features {
                            login: LoginFeature {
                                simple: true,
                                ..Default::default()
                            },
                            teams: true,
                        },
                        &con,
                    );
                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                        .await?;
                    driver.get(format!("http://localhost:{}", port)).await?;

                    assert!(driver
                        .find_element(By::Css("[aria-label='Switch administration mode']"))
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
    use thirtyfour::components::select::SelectElement;

    use super::*;

    #[async_test]
    async fn as_user_with_capability() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let con = con.lock().await;
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[],
                        &[Capability::ShortcutsWrite],
                        &con,
                    );
                    global_features(
                        &Features {
                            login: LoginFeature {
                                simple: true,
                                ..Default::default()
                            },
                            teams: true,
                        },
                        &con,
                    );

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
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
    async fn as_user_with_team_capability() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let con = con.lock().await;
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[("", &[TeamCapability::ShortcutsWrite], 0)],
                        &[],
                        &con,
                    );
                    global_features(
                        &Features {
                            login: LoginFeature {
                                simple: true,
                                ..Default::default()
                            },
                            teams: true,
                        },
                        &con,
                    );

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
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
                    let con = con.lock().await;
                    team("team1", "team1", true, true, &con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[],
                        &[Capability::ShortcutsWrite],
                        &con,
                    );
                    global_features(
                        &Features {
                            login: LoginFeature {
                                simple: true,
                                ..Default::default()
                            },
                            teams: true,
                        },
                        &con,
                    );

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                        .await?;
                    assert_create_shortcut_ok(driver, "team1", port).await;

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
            .get(format!("http://localhost:{}", port))
            .await
            .unwrap();

        driver
            .find_element(By::Css("[aria-label='Switch administration mode']"))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        driver
            .find_element(By::Css("[name='shortcut']"))
            .await
            .unwrap()
            .send_keys("jeanLuc")
            .await
            .unwrap();
        driver
            .find_element(By::Css("[name='url']"))
            .await
            .unwrap()
            .send_keys(format!("http://localhost:{}/aShortcut", port))
            .await
            .unwrap();

        let input = driver.find_element(By::Css("[name='team']")).await.unwrap();
        let select = SelectElement::new(&input).await.unwrap();
        select.select_by_value(team).await.unwrap();

        driver
            .find_element(By::Css("[aria-label='Add shortcut']"))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        let article = driver
            .find_element(By::Css("[role='listitem']"))
            .await
            .unwrap();
        assert_eq!(
            article.text().await.unwrap(),
            format!("jeanLuc http://localhost:{}/aShortcut {}NEW", port, team)
        );

        assert_eq!(
            article.get_property("href").await.unwrap(),
            Some(format!("http://localhost:{}/jeanLuc?no_redirect", port))
        );

        assert_eq!(
            driver
                .find_element(By::Css("[name='shortcut']"))
                .await
                .unwrap()
                .get_property("value")
                .await
                .unwrap(),
            Some("".to_owned())
        );
        assert_eq!(
            driver
                .find_element(By::Css("[name='url']"))
                .await
                .unwrap()
                .get_property("value")
                .await
                .unwrap(),
            Some("".to_owned())
        );

        assert_eq!(
            driver
                .find_element(By::Css("[name='team']"))
                .await
                .unwrap()
                .get_property("value")
                .await
                .unwrap(),
            Some("".to_owned())
        );

        driver.refresh().await.unwrap();

        let article = driver
            .find_element(By::Css("[role='listitem']"))
            .await
            .unwrap();
        assert_eq!(
            article.text().await.unwrap(),
            format!(
                "jeanLuc http://localhost:{}/aShortcut{}{}",
                port, spacer, team
            )
        );
    }
}
