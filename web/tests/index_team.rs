use diesel::PgConnection;
use go_web::teams::TeamCapability;
use go_web::users::Capability;
use rocket::futures::FutureExt;
use rocket::http::Status;
use rocket::tokio::sync::Mutex;
use rocket::{async_test, http};
use thirtyfour::error::WebDriverError;
use thirtyfour::prelude::*;
use thirtyfour_testing_library_ext::{Screen, By as ByExt, TextMatch};

mod utils;
use go_web::guards::SESSION_COOKIE;

use utils::*;

#[async_test]
async fn as_user() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut conn = con.lock().await;
                team("slug1", "team1", false, true, &mut conn);
                shortcut(
                    "newShortcut",
                    &host(port, "/looped"),
                    "slug1",
                    &mut conn,
                );
                shortcut(
                    "newShortcut2",
                    &host(port, "/claude"),
                    "slug1",
                    &mut conn,
                );

                user("some_mail@mail.com", "pwd", &[], &[], &mut conn);

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;
                driver
                    .get(host(port, "/go/teams/slug1"))
                    .await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                let shortcut_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/Shortcut list/".to_string()))).await?;
                let scoped_screen = screen.within(shortcut_list);
                let articles = scoped_screen.find_all(ByExt::role("listitem")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("newShortcut {} slug1", host(port, "/looped"))
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
async fn list_user() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                team("slug1", "team1", false, true, &mut con);
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("slug1", &[], 0, true)],
                    &[Capability::TeamsWrite],
                    &mut con,
                );
                user(
                    "another_mail@mail.com",
                    "pwd",
                    &[("slug1", &[], 0, true)],
                    &[],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;

                driver
                    .get(host(port, "/go/teams/slug1"))
                    .await?;

                // TODO ordering
                let expected_users = vec!["some_mail@mail.com", "another_mail@mail.com"];

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                let users = screen.find_all(ByExt::role("heading")).await?;
                for i in 0..expected_users.len() {
                    assert_eq!(users[i].text().await.unwrap(), expected_users[i]);
                }

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

mod edit_team {
    use super::*;

    #[async_test]
    async fn without_capabilities_is_not_authorized() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut conn = con.lock().await;
                    team("slug1", "team1", false, true, &mut conn);
                    shortcut(
                        "newShortcut",
                        &host(port, "/looped"),
                        "slug1",
                        &mut conn,
                    );
                    shortcut(
                        "newShortcut2",
                        &host(port, "/claude"),
                        "slug1",
                        &mut conn,
                    );

                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[("slug1", &[], 0, true)],
                        &[],
                        &mut conn,
                    );

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                        .await?;
                    driver
                        .get(host(port, "/go/teams/slug1"))
                        .await?;

                    let screen = Screen::build_with_testing_library(driver.clone()).await?;
                    assert!(screen
                        .query(ByExt::role("region").name(TextMatch::Exact("Team editor".to_string())))
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
    async fn as_teamate_can_edit() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut conn = con.lock().await;
                    team("slug1", "team1", false, true, &mut conn);
                    shortcut(
                        "newShortcut",
                        &host(port, "/looped"),
                        "slug1",
                        &mut conn,
                    );
                    shortcut(
                        "newShortcut2",
                        &host(port, "/claude"),
                        "slug1",
                        &mut conn,
                    );

                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                        &[],
                        &mut conn,
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
    async fn as_admin_can_edit() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut conn = con.lock().await;
                    team("slug1", "team1", false, true, &mut conn);
                    shortcut(
                        "newShortcut",
                        &host(port, "/looped"),
                        "slug1",
                        &mut conn,
                    );
                    shortcut(
                        "newShortcut2",
                        &host(port, "/claude"),
                        "slug1",
                        &mut conn,
                    );

                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[],
                        &[Capability::TeamsWrite, Capability::TeamsCreateWithValidation, Capability::UsersTeamsRead],
                        &mut conn,
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
            .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .await?;
        driver
            .get(host(port, "/go/teams/slug1"))
            .await?;

        let screen = Screen::build_with_testing_library(driver.clone()).await?;
        let title = screen.find(ByExt::label_text("Title")).await?;
        assert_eq!(
            title.prop("value").await?,
            Some("team1".to_string())
        );
        title.send_keys("2").await?;

        let is_private = screen.find(ByExt::label_text("Private")).await?;
        assert_eq!(
            is_private.prop("checked").await?,
            Some("false".to_string())
        );
        is_private.click().await?;

        if admin {
            let is_accepted = screen.find(ByExt::label_text("Enable")).await?;
            assert_eq!(
                is_accepted.prop("checked").await?,
                Some("true".to_string())
            );
            is_accepted.click().await?;
        }

        screen
            .find(ByExt::text("Save"))
            .await?
            .click()
            .await?;

        driver
            .get(host(port, "/go/teams/slug1"))
            .await?;

        let screen = Screen::build_with_testing_library(driver.clone()).await?;
        assert_eq!(
            screen
                .find(ByExt::label_text("Title"))
                .await?
                .prop("value")
                .await?,
            Some("team12".to_string())
        );

        assert_eq!(
            screen
                .find(ByExt::label_text("Private"))
                .await?
                .prop("checked")
                .await?,
            Some("true".to_string())
        );

        if admin {
            assert_eq!(
                screen
                    .find(ByExt::label_text("Enable"))
                    .await?
                    .prop("checked")
                    .await?,
                Some("false".to_string())
            );
        }

        Ok(())
    }
}

mod edit_user_team_link {
    use super::*;

    #[async_test]
    async fn edit_user_capability_as_admin() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("slug1", "team1", false, true, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[],
                        &[Capability::TeamsWrite],
                        &mut con,
                    );
                    user(
                        "another_mail@mail.com",
                        "pwd",
                        &[("slug1", &[], 0, true)],
                        &[],
                        &mut con,
                    );

                    toggle_capability(driver, "another_mail@mail.com", port).await;

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn edit_user_capability_as_teamate() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("slug1", "team1", false, true, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                        &[],
                        &mut con,
                    );
                    user(
                        "another_mail@mail.com",
                        "pwd",
                        &[("slug1", &[], 0, true)],
                        &[],
                        &mut con,
                    );

                    toggle_capability(driver, "another_mail@mail.com", port).await;

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    async fn toggle_capability(driver: &WebDriver, mail: &str, port: u16) {
        driver
            .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .await
            .unwrap();

        driver
            .get(host(port, "/go/teams/slug1"))
            .await
            .unwrap();

        let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
        let user_list = screen.find(ByExt::role("list").name(TextMatch::Exact("User list".to_string()))).await.unwrap();
        let user_rows = screen.within(user_list).find_all(ByExt::role("listitem")).await.unwrap();

        // Find the user row matching the given mail
        let mut user_row = None;
        for row in &user_rows {
            if let Ok(text) = row.text().await {
                if text.contains(mail) {
                    user_row = Some(row);
                    break;
                }
            }
        }
        let user_row = user_row.expect("User row not found");

        user_row
            .click()
            .await
            .unwrap();

        let shortcuts_write_switch = screen.within(user_row.clone()).find(ByExt::label_text(&TeamCapability::ShortcutsWrite.to_string())).await.unwrap();

        assert_eq!(
            shortcuts_write_switch.prop("checked").await.unwrap().unwrap(),
            "false"
        );

        shortcuts_write_switch.click().await.unwrap();
        assert_eq!(
            shortcuts_write_switch.prop("checked").await.unwrap().unwrap(),
            "true"
        );

        driver
            .get(host(port, "/go/teams/slug1"))
            .await
            .unwrap();

        let user_list = screen.find(ByExt::role("list").name(TextMatch::Exact("User list".to_string()))).await.unwrap();
        let user_rows = screen.within(user_list).find_all(ByExt::role("listitem")).await.unwrap();

        // Find the user row matching the given mail
        let mut user_row = None;
        for row in &user_rows {
            if let Ok(text) = row.text().await {
                if text.contains(mail) {
                    user_row = Some(row);
                    break;
                }
            }
        }
        let user_row = user_row.expect("User row not found");

        user_row
            .click()
            .await
            .unwrap();

        let shortcuts_write_switch = screen.within(user_row.clone()).find(ByExt::label_text(&TeamCapability::ShortcutsWrite.to_string())).await.unwrap();

        assert_eq!(
            shortcuts_write_switch.prop("checked").await.unwrap().unwrap(),
            "true"
        );

        shortcuts_write_switch.scroll_into_view().await.unwrap();
        shortcuts_write_switch.click().await.unwrap();
        assert_eq!(
            shortcuts_write_switch.prop("checked").await.unwrap().unwrap(),
            "false"
        );
    }
}

mod kick_user {
    use super::*;

    #[async_test]
    async fn as_admin() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("slug1", "team1", false, true, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[],
                        &[Capability::TeamsWrite],
                        &mut con,
                    );
                    user(
                        "another_mail@mail.com",
                        "pwd",
                        &[("slug1", &[], 0, true)],
                        &[],
                        &mut con,
                    );

                    kick(driver, "another_mail@mail.com", port).await;

                    let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
                    let user = screen.query(ByExt::text("another_mail@mail.com")).await.unwrap();
                    assert!(user.is_none());

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn as_teamate() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("slug1", "team1", false, true, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                        &[],
                        &mut con,
                    );
                    user(
                        "another_mail@mail.com",
                        "pwd",
                        &[("slug1", &[], 0, true)],
                        &[],
                        &mut con,
                    );

                    kick(driver, "another_mail@mail.com", port).await;

                    let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
                    let user = screen.query(ByExt::text("another_mail@mail.com")).await.unwrap();
                    assert!(user.is_none());
                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn as_self() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("slug1", "team1", false, true, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                        &[],
                        &mut con,
                    );

                    kick(driver, "some_mail@mail.com", port).await;

                    let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
                    let team_editor = screen.query(ByExt::display_value("team1")).await.unwrap();
                    assert!(team_editor.is_none());
                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    
    }

    async fn kick(driver: &WebDriver, mail: &str, port: u16) {
        driver
            .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .await
            .unwrap();

        driver
            .get(host(port, "/go/teams/slug1"))
            .await
            .unwrap();

        let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
        let user_list = screen.find(ByExt::role("list").name(TextMatch::Exact("User list".to_string()))).await.unwrap();
        let user_rows = screen.within(user_list).find_all(ByExt::role("listitem")).await.unwrap();
        // Find the user row matching the given mail
        let mut user_row = None;
        for row in &user_rows {
            if let Ok(text) = row.text().await {
                if text.contains(mail) {
                    user_row = Some(row);
                    break;
                }
            }
        }
        let user_row = user_row.expect("User row not found");        user_row.click().await.unwrap();

        let kick_button = screen
            .find(ByExt::role("button").name(TextMatch::Exact("Kick user".to_string())))
            .await
            .unwrap();

        kick_button.wait_until().displayed().await.unwrap();

        assert_eq!("Kick", kick_button.text().await.unwrap());
        kick_button.click().await.unwrap();       
    }

}

mod accept_user {
    use super::*;

    #[async_test]
    async fn already_accepted_cant_be_reaccepted() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("slug1", "team1", false, true, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[],
                        &[Capability::TeamsWrite],
                        &mut con,
                    );
                    user(
                        "another_mail@mail.com",
                        "pwd",
                        &[("slug1", &[], 0, true)],
                        &[],
                        &mut con,
                    );

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                        .await
                        .unwrap();

                    driver
                        .get(host(port, "/go/teams/slug1"))
                        .await
                        .unwrap();

                    let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
                    assert!(screen
                        .query(ByExt::role("button").name(TextMatch::Exact("Accept candidature".to_string())))
                        .await.unwrap()
                        .is_none());

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn as_admin() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("slug1", "team1", false, true, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[],
                        &[Capability::TeamsWrite, Capability::UsersTeamsRead],
                        &mut con,
                    );
                    user(
                        "another_mail@mail.com",
                        "pwd",
                        &[("slug1", &[], 0, false)],
                        &[],
                        &mut con,
                    );

                    accept_candidature(driver,"another_mail@mail.com", port).await;

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn as_teamate() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("slug1", "team1", false, true, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                        &[],
                        &mut con,
                    );
                    user(
                        "another_mail@mail.com",
                        "pwd",
                        &[("slug1", &[], 0, false)],
                        &[],
                        &mut con,
                    );

                    accept_candidature(driver,"another_mail@mail.com", port,).await;

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    async fn accept_candidature(driver: &WebDriver,mail: &str,  port: u16) {
        driver
            .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .await
            .unwrap();

        driver
            .get(host(port, "/go/teams/slug1"))
            .await
            .unwrap();

        let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
        let user_list = screen.find(ByExt::role("list").name(TextMatch::Exact("User list".to_string()))).await.unwrap();
        let user_rows = screen.within(user_list).find_all(ByExt::role("listitem")).await.unwrap();

        // Find the user row matching the given mail
        let mut user_row = None;
        for row in &user_rows {
            if let Ok(text) = row.text().await {
                if text.contains(mail) {
                    user_row = Some(row);
                    break;
                }
            }
        }
        let user_row = user_row.expect("User row not found");

        user_row
            .click()
            .await
            .unwrap();

        let accept_button = screen
            .find(ByExt::role("button").name(TextMatch::Exact("Accept candidature".to_string())))
            .await
            .unwrap();

        accept_button.wait_until().displayed().await.unwrap();

        assert_eq!("Accept candidature", accept_button.text().await.unwrap());
        accept_button.click().await.unwrap();

        assert!(screen
            .query(ByExt::role("button").name(TextMatch::Exact("Accept candidature".to_string())))
            .await.unwrap()
            .is_none());

        driver
            .get(host(port, "/go/teams/slug1"))
            .await
            .unwrap();
        let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
        assert!(screen
            .query(ByExt::role("button").name(TextMatch::Exact("Accept candidature".to_string())))
            .await.unwrap()
            .is_none());
    }
}

mod controller {
    use super::*;

    #[test]
    fn as_unknown_user_is_not_allowed() {
        let (client, mut conn) = launch_with("");

        user("some_mail@mail.com", "pwd", &[], &[], &mut conn);

        let response = client.get("/go/teams/slug1").dispatch();

        assert_eq!(response.status(), Status::Unauthorized);

        let response = client
            .get("/go/teams/slug1")
            .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
            .dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[test]
    fn unknown_team_return_404() {
        let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");

        user("some_mail@mail.com", "pwd", &[], &[], &mut conn);

        let response = client
            .get("/go/teams/slug1")
            .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn private_team_return_404() {
        let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
        team("slug1", "team1", true, true, &mut conn);

        user("some_mail@mail.com", "pwd", &[], &[], &mut conn);

        let response = client
            .get("/go/teams/slug1")
            .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }

    mod kick {
        use super::*;

        #[test]
        fn without_capability_is_not_allowed() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", true, true, &mut conn);

            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[], 0, true)],
                &[],
                &mut conn,
            );
            user("other_mail@mail.com", "pwd", &[("slug1", &[], 0, true)], &[], &mut conn);


            let response = client
                .delete("/go/teams/slug1/users/other_mail@mail.com")
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .delete("/go/teams/slug1/users/other_mail@mail.com")
                .cookie(http::Cookie::new(SESSION_COOKIE, "other_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .delete("/go/teams/slug1/users/other_mail@mail.com")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);
        }

        #[test]
        fn as_admin() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", true, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::TeamsWrite],
                &mut conn,
            );
            user("other_mail@mail.com", "pwd", &[("slug1", &[], 0, true)], &[], &mut conn);

            let response = client
                .delete("/go/teams/slug1/users/other_mail@mail.com")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);

            assert!(get_user_team_links("other_mail@mail.com", &mut conn).is_empty());
        }

        #[test]
        fn as_teamate() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", true, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                &[],
                &mut conn,
            );
            user("other_mail@mail.com", "pwd",                 &[("slug1", &[], 0, true)], &[], &mut conn);

            let response = client
                .delete("/go/teams/slug1/users/other_mail@mail.com")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);

            assert!(get_user_team_links("other_mail@mail.com", &mut conn).is_empty());
        }

        #[test]
        fn as_self() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", true, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[], 0, true)],
                &[],
                &mut conn,
            );

            let response = client
                .delete("/go/teams/slug1/users/some_mail@mail.com")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);

            assert!(get_user_team_links("some_mail@mail.com", &mut conn).is_empty());
        }
    }

    mod accept_candidature {
        use super::*;

        #[test]
        fn without_capability_is_not_allowed() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", true, true, &mut conn);

            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[], 0, true)],
                &[],
                &mut conn,
            );
            user(
                "other_mail@mail.com",
                "pwd",
                &[("slug1", &[], 0, false)],
                &[],
                &mut conn,
            );

            let response = client
                .put("/go/teams/slug1/users/some_mail@mail.com/is_accepted/true")
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .put("/go/teams/slug1/users/some_mail@mail.com/is_accepted/true")
                .cookie(http::Cookie::new(SESSION_COOKIE, "other_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .put("/go/teams/slug1/users/some_mail@mail.com/is_accepted/true")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);
        }

        #[test]
        fn as_admin() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", true, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::TeamsWrite],
                &mut conn,
            );
            user(
                "other_mail@mail.com",
                "pwd",
                &[("slug1", &[], 0, false)],
                &[],
                &mut conn,
            );
            let response = client
                .put("/go/teams/slug1/users/other_mail@mail.com/is_accepted/true")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);

            assert!(
                get_user_team_links("other_mail@mail.com", &mut conn)
                    .first()
                    .unwrap()
                    .is_accepted
            );
        }

        #[test]
        fn as_teamate() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", true, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                &[],
                &mut conn,
            );
            user(
                "other_mail@mail.com",
                "pwd",
                &[("slug1", &[], 0, false)],
                &[],
                &mut conn,
            );

            let response = client
                .put("/go/teams/slug1/users/other_mail@mail.com/is_accepted/true")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);

            assert!(
                get_user_team_links("other_mail@mail.com", &mut conn)
                    .first()
                    .unwrap()
                    .is_accepted
            );
        }
    }

    mod put_user_capability {
        use super::*;

        #[test]
        fn without_capability_is_not_allowed() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", true, true, &mut conn);

            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[], 0, true)],
                &[],
                &mut conn,
            );

            let response = client
                .put("/go/teams/slug1/users/some_mail@mail.com/capabilities/TeamsWrite")
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .put("/go/teams/slug1/users/some_mail@mail.com/capabilities/TeamsWrite")
                .cookie(http::Cookie::new(SESSION_COOKIE, "other_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .put("/go/teams/slug1/users/some_mail@mail.com/capabilities/TeamsWrite")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);
        }

        #[test]
        fn as_admin() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", true, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::TeamsWrite],
                &mut conn,
            );
            user(
                "other_mail@mail.com",
                "pwd",
                &[("slug1", &[], 0, true)],
                &[],
                &mut conn,
            );

            let response = client
                .put("/go/teams/slug1/users/other_mail@mail.com/capabilities/TeamsWrite")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);

            assert_eq!(
                vec![TeamCapability::TeamsWrite],
                get_user_team_links("other_mail@mail.com", &mut conn)
                    .first()
                    .unwrap()
                    .capabilities
            );
        }

        #[test]
        fn as_teamate() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", true, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                &[],
                &mut conn,
            );
            user(
                "other_mail@mail.com",
                "pwd",
                &[("slug1", &[], 0, true)],
                &[],
                &mut conn,
            );

            let response = client
                .put("/go/teams/slug1/users/other_mail@mail.com/capabilities/TeamsWrite")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);

            assert_eq!(
                vec![TeamCapability::TeamsWrite],
                get_user_team_links("other_mail@mail.com", &mut conn)
                    .first()
                    .unwrap()
                    .capabilities
            );
        }
    }

    mod delete_user_capability {
        use super::*;

        #[test]
        fn without_capability_is_not_allowed() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", true, true, &mut conn);

            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[], 0, true)],
                &[],
                &mut conn,
            );

            let response = client
                .delete("/go/teams/slug1/users/some_mail@mail.com/capabilities/TeamsWrite")
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .delete("/go/teams/slug1/users/some_mail@mail.com/capabilities/TeamsWrite")
                .cookie(http::Cookie::new(SESSION_COOKIE, "other_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .delete("/go/teams/slug1/users/some_mail@mail.com/capabilities/TeamsWrite")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);
        }

        #[test]
        fn as_admin() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", true, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::TeamsWrite],
                &mut conn,
            );
            user(
                "other_mail@mail.com",
                "pwd",
                &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                &[],
                &mut conn,
            );

            let response = client
                .delete("/go/teams/slug1/users/other_mail@mail.com/capabilities/TeamsWrite")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);

            assert!(get_user_team_links("other_mail@mail.com", &mut conn)
                .first()
                .unwrap()
                .capabilities
                .is_empty(),);
        }

        #[test]
        fn as_teamate() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", true, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                &[],
                &mut conn,
            );
            user(
                "other_mail@mail.com",
                "pwd",
                &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                &[],
                &mut conn,
            );

            let response = client
                .delete("/go/teams/slug1/users/other_mail@mail.com/capabilities/TeamsWrite")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);

            assert!(get_user_team_links("other_mail@mail.com", &mut conn)
                .first()
                .unwrap()
                .capabilities
                .is_empty(),);
        }
    }
}
