use diesel::PgConnection;
use go_web::models::teams::TeamCapability;
use go_web::models::users::Capability;
use rocket::futures::FutureExt;
use rocket::http::Status;
use rocket::tokio::sync::Mutex;
use rocket::{async_test, http};
use serde_json::json;
use thirtyfour::error::WebDriverError;
use thirtyfour::prelude::*;

mod utils;
use go_web::guards::SESSION_COOKIE;
use go_web::models::settings::{Features, LoginFeature};
use utils::*;

#[test]
fn feature_team_is_required() {
    let (client, _conn) = launch_with("");
    let response = client.get("/go/teams/slug1").dispatch();

    assert_eq!(response.status(), Status::Conflict);
}

#[test]
fn show_public_team_require_user_with_capabilities() {
    let (client, conn) = launch_with("");
    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                ..Default::default()
            },
            teams: true,
        },
        &conn,
    );
    user("some_mail@mail.com", "pwd", &[], &[], &conn);

    let response = client.get("/go/teams/slug1").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);

    let response = client
        .get("/go/teams/slug1")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn show_team_that_do_not_exit_return_404() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                ..Default::default()
            },
            teams: true,
        },
        &conn,
    );
    user(
        "some_mail@mail.com",
        "pwd",
        &[],
        &[Capability::TeamsRead],
        &conn,
    );

    let response = client
        .get("/go/teams/slug1")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn show_private_team_user_not_in_return_404() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    team("slug1", "team1", true, true, &conn);
    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                ..Default::default()
            },
            teams: true,
        },
        &conn,
    );
    user(
        "some_mail@mail.com",
        "pwd",
        &[],
        &[Capability::TeamsRead],
        &conn,
    );

    let response = client
        .get("/go/teams/slug1")
        .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[async_test]
async fn show_team_with_shortcuts() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let conn = con.lock().await;
                team("slug1", "team1", false, true, &conn);
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/looped", port),
                    "slug1",
                    &conn,
                );
                shortcut(
                    "newShortcut2",
                    &format!("http://localhost:{}/claude", port),
                    "slug1",
                    &conn,
                );
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            ..Default::default()
                        },
                        teams: true,
                    },
                    &conn,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[],
                    &[Capability::TeamsRead],
                    &conn,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                    .await?;
                driver
                    .get(format!("http://localhost:{}/go/teams/slug1", port))
                    .await?;

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("newShortcut http://localhost:{}/looped slug1", port)
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
async fn show_team_user_can_edit() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let conn = con.lock().await;
                team("slug1", "team1", false, true, &conn);
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/looped", port),
                    "slug1",
                    &conn,
                );
                shortcut(
                    "newShortcut2",
                    &format!("http://localhost:{}/claude", port),
                    "slug1",
                    &conn,
                );
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            ..Default::default()
                        },
                        teams: true,
                    },
                    &conn,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                    &[Capability::TeamsRead],
                    &conn,
                );

                assert_user_can_update_team(driver, port, false).await?;

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn show_team_admin_can_edit() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let conn = con.lock().await;
                team("slug1", "team1", false, true, &conn);
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/looped", port),
                    "slug1",
                    &conn,
                );
                shortcut(
                    "newShortcut2",
                    &format!("http://localhost:{}/claude", port),
                    "slug1",
                    &conn,
                );
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            ..Default::default()
                        },
                        teams: true,
                    },
                    &conn,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[],
                    &[Capability::TeamsRead, Capability::TeamsWrite],
                    &conn,
                );

                assert_user_can_update_team(driver, port, true).await?;

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

async fn assert_user_can_update_team(
    driver: &WebDriver,
    port: u16,
    admin: bool,
) -> Result<(), WebDriverError> {
    driver
        .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
        .await?;
    driver
        .get(format!("http://localhost:{}/go/teams/slug1", port))
        .await?;

    let title = driver.find_element(By::Css("[name='title']")).await?;
    assert_eq!(
        title.get_property("value").await?,
        Some("team1".to_string())
    );
    title.send_keys("2").await?;

    let is_private = driver.find_element(By::Css("[name='is_private']")).await?;
    assert_eq!(
        is_private.get_property("checked").await?,
        Some("false".to_string())
    );
    is_private.click().await?;

    if admin {
        let is_accepted = driver.find_element(By::Css("[name='is_accepted']")).await?;
        assert_eq!(
            is_accepted.get_property("checked").await?,
            Some("true".to_string())
        );
        is_accepted.click().await?;
    }

    driver
        .find_element(By::Css("[type='submit']"))
        .await?
        .click()
        .await?;

    driver
        .get(format!("http://localhost:{}/go/teams/slug1", port))
        .await?;

    assert_eq!(
        driver
            .find_element(By::Css("[name='title']"))
            .await?
            .get_property("value")
            .await?,
        Some("team12".to_string())
    );

    assert_eq!(
        driver
            .find_element(By::Css("[name='is_private']"))
            .await?
            .get_property("checked")
            .await?,
        Some("true".to_string())
    );

    if admin {
        assert_eq!(
            driver
                .find_element(By::Css("[name='is_accepted']"))
                .await?
                .get_property("checked")
                .await?,
            Some("false".to_string())
        );
    }

    Ok(())
}
