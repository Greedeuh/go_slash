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

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
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

                let expected_users = vec!["another_mail@mail.com", "some_mail@mail.com"];

                let users = driver
                    .find_elements(By::Css("[role='listitem'] h2"))
                    .await
                    .unwrap();
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

                    assert!(driver
                        .find_element(By::Css("[aria-label='Team editor']"))
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
                        &[Capability::TeamsWrite],
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
            .get(host(port, "/go/teams/slug1"))
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

                    toggle_capability(driver, port).await;

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

                    toggle_capability(driver, port).await;

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    async fn toggle_capability(driver: &WebDriver, port: u16) {
        driver
            .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .await
            .unwrap();

        driver
            .get(host(port, "/go/teams/slug1"))
            .await
            .unwrap();

        let user = driver
            .find_element(By::Css("[aria-label='User list'] [role='listitem']"))
            .await
            .unwrap();
        user.click().await.unwrap();

        let switchs = user
            .find_elements(By::Css("[role='switch']"))
            .await
            .unwrap();
        let switch = switchs.first().unwrap();
        let switchs_label = user.find_elements(By::Tag("label")).await.unwrap();
        let switch_label = switchs_label.first().unwrap();

        switch_label.wait_until().displayed().await.unwrap();
        assert_eq!(
            switch_label.text().await.unwrap(),
            TeamCapability::ShortcutsWrite.to_string()
        );
        assert_eq!(
            switch.get_property("checked").await.unwrap().unwrap(),
            "false"
        );

        switch.click().await.unwrap();
        assert_eq!(
            switch.get_property("checked").await.unwrap().unwrap(),
            "true"
        );

        driver
            .get(host(port, "/go/teams/slug1"))
            .await
            .unwrap();

        driver
            .find_element(By::Css("[aria-label='User list'] [role='listitem']"))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        let switchs = driver
            .find_elements(By::Css(
                "[aria-label='User list'] [role='listitem'] [role='switch']",
            ))
            .await
            .unwrap();
        let switch = &switchs[0];

        switch.wait_until().displayed().await.unwrap();
        assert_eq!(
            switch.get_property("checked").await.unwrap().unwrap(),
            "true"
        );

        switch.click().await.unwrap();
        assert_eq!(
            switch.get_property("checked").await.unwrap().unwrap(),
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

                    kick(driver, port).await;

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

                    kick(driver, port).await;

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    async fn kick(driver: &WebDriver, port: u16) {
        driver
            .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .await
            .unwrap();

        driver
            .get(host(port, "/go/teams/slug1"))
            .await
            .unwrap();

        driver
            .find_element(By::Css("[role='listitem'] h2"))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        let kick_button = driver
            .find_element(By::Css("[aria-label='Kick user']"))
            .await
            .unwrap();

        kick_button.wait_until().displayed().await.unwrap();

        assert_eq!("Kick", kick_button.text().await.unwrap());
        kick_button.click().await.unwrap();

        assert!(driver
            .find_element(By::Css("[role='listitem'] h2"))
            .await
            .is_err());

        driver
            .get(host(port, "/go/teams/slug1"))
            .await
            .unwrap();
        assert!(driver
            .find_element(By::Css("[role='listitem'] h2"))
            .await
            .is_err());
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

                    assert!(driver
                        .find_element(By::Css("[aria-label='Accept candidature']"))
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
                        &[("slug1", &[], 0, false)],
                        &[],
                        &mut con,
                    );

                    accept_candidature(driver, port).await;

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

                    accept_candidature(driver, port).await;

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    async fn accept_candidature(driver: &WebDriver, port: u16) {
        driver
            .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .await
            .unwrap();

        driver
            .get(host(port, "/go/teams/slug1"))
            .await
            .unwrap();

        driver
            .find_element(By::Css("[role='listitem'] h2"))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        let accept_button = driver
            .find_element(By::Css("[aria-label='Accept candidature']"))
            .await
            .unwrap();

        accept_button.wait_until().displayed().await.unwrap();

        assert_eq!("Accept candidature", accept_button.text().await.unwrap());
        accept_button.click().await.unwrap();

        assert!(driver
            .find_element(By::Css("[aria-label='Accept candidature']"))
            .await
            .is_err());

        driver
            .get(host(port, "/go/teams/slug1"))
            .await
            .unwrap();
        assert!(driver
            .find_element(By::Css("[aria-label='Accept candidature']"))
            .await
            .is_err());
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

            let response = client
                .delete("/go/teams/slug1/users/some_mail@mail.com")
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .delete("/go/teams/slug1/users/some_mail@mail.com")
                .cookie(http::Cookie::new(SESSION_COOKIE, "other_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .delete("/go/teams/slug1/users/some_mail@mail.com")
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
            user("other_mail@mail.com", "pwd", &[], &[], &mut conn);

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
            user("other_mail@mail.com", "pwd", &[], &[], &mut conn);

            let response = client
                .delete("/go/teams/slug1/users/other_mail@mail.com")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);

            assert!(get_user_team_links("other_mail@mail.com", &mut conn).is_empty());
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
