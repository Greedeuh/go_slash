use diesel::PgConnection;
use go_web::models::teams::{Team, TeamCapability};
use go_web::models::users::{Capability, UserTeam};
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
async fn as_admin_accept_team() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                team("slug1", "team1", false, false, &mut con);
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("slug1", &[], 1, true)],
                    &[Capability::TeamsWrite],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;

                driver
                    .get(host(port, "/go/teams"))
                    .await?;

                assert!(driver
                    .find(By::Css("button[aria-label='Accept team']"))
                    .await
                    .is_err());

                driver
                    .find(By::Css("button[aria-label='Administrate']"))
                    .await?
                    .click()
                    .await?;

                assert!(dbg!(
                    driver
                        .find(By::Css("[role='listitem']"))
                        .await?
                        .text()
                        .await?
                )
                .starts_with("team1"));

                let accept_btn = driver
                    .find(By::Css("button[aria-label='Accept team']"))
                    .await?;
                accept_btn.click().await?;
                assert!(driver
                    .find(By::Css("button[aria-label='Accept team']"))
                    .await
                    .is_err());

                driver
                    .get(host(port, "/go/teams"))
                    .await?;

                driver
                    .find(By::Css("button[aria-label='Administrate']"))
                    .await?
                    .click()
                    .await?;

                assert!(driver
                    .find(By::Css("button[aria-label='Accept team']"))
                    .await
                    .is_err());

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

mod delete {
    use super::*;

    #[async_test]
    async fn as_admin() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("slug1", "team1", false, false, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[("slug1", &[], 1, true)],
                        &[Capability::TeamsWrite],
                        &mut con,
                    );

                    delete_team(driver, port).await;

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn as_teamate_without_capability_is_not_allowed() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("slug1", "team1", false, false, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[("slug1", &[], 1, true)],
                        &[],
                        &mut con,
                    );

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                        .await
                        .unwrap();

                    driver
                        .get(host(port, "/go/teams"))
                        .await
                        .unwrap();

                    assert!(driver
                        .find(By::Css("button[aria-label='Delete team']"))
                        .await
                        .is_err());

                    assert!(driver
                        .find(By::Css("button[aria-label='Administrate']"))
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
    async fn as_teamate_its_only_allowed_for_team_with_capability() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("slug1", "team1", false, false, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[
                            ("slug1", &[TeamCapability::TeamsWrite], 1, true),
                            ("", &[TeamCapability::TeamsWrite], 1, false),
                        ],
                        &[],
                        &mut con,
                    );

                    driver
                        .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                        .await
                        .unwrap();

                    driver
                        .get(host(port, "/go/teams"))
                        .await
                        .unwrap();

                    driver
                        .find(By::Css("button[aria-label='Administrate']"))
                        .await?
                        .click()
                        .await?;
                    assert!(driver
                        .find(By::Css(
                            "[href='/go/teams/slug1'] [aria-label='Delete team']",
                        ))
                        .await
                        .is_ok());
                    assert!(driver
                        .find(By::Css("[href='/go/teams/'] [aria-label='Delete team']"))
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
    async fn as_teamate() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    team("slug1", "team1", false, false, &mut con);
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[("slug1", &[TeamCapability::TeamsWrite], 1, true)],
                        &[],
                        &mut con,
                    );

                    delete_team(driver, port).await;

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    async fn delete_team(driver: &WebDriver, port: u16) {
        driver
            .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .await
            .unwrap();

        driver
            .get(host(port, "/go/teams"))
            .await
            .unwrap();

        assert!(driver
            .find(By::Css("button[aria-label='Delete team']"))
            .await
            .is_err());

        driver
            .find(By::Css("button[aria-label='Administrate']"))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        assert!(dbg!(driver
            .find(By::Css("[role='listitem']"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap())
        .starts_with("team1"));

        let delete_btn = driver
            .find(By::Css("button[aria-label='Delete team']"))
            .await
            .unwrap();
        delete_btn.click().await.unwrap();

        assert!(!dbg!(driver
            .find(By::Css("[role='listitem']"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap())
        .starts_with("team1"));

        driver
            .get(host(port, "/go/teams"))
            .await
            .unwrap();

        assert!(!dbg!(driver
            .find(By::Css("[role='listitem']"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap())
        .starts_with("team1"));
    }
}

mod create {
    use super::*;

    #[async_test]
    async fn as_admin() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[],
                        &[Capability::TeamsWrite],
                        &mut con,
                    );

                    create_team(driver, port).await?;

                    let create_dialog = driver.find(By::Css("[role='dialog']")).await?;
                    assert!(dbg!(
                        create_dialog
                            .find(By::Css("[aria-label='Create team result']"))
                            .await?
                            .text()
                            .await?
                    )
                    .starts_with("Success !"));

                    dialog_close_then_open(driver).await;

                    assert_create_form_is_empty(driver).await;

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    #[async_test]
    async fn as_user() {
        in_browser(
            "some_session_id: some_mail@mail.com",
            |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
                async move {
                    let mut con = con.lock().await;
                    user(
                        "some_mail@mail.com",
                        "pwd",
                        &[],
                        &[Capability::TeamsWriteWithValidation],
                        &mut con,
                    );

                    create_team(driver, port).await?;

                    let create_dialog = driver.find(By::Css("[role='dialog']")).await?;
                    assert!(dbg!(
                        create_dialog
                            .find(By::Css("[aria-label='Create team result']"))
                            .await?
                            .text()
                            .await?
                    )
                    .starts_with("Success ! Your Admins will now have to validate your team."));

                    dialog_close_then_open(driver).await;

                    assert_create_form_is_empty(driver).await;

                    Ok(())
                }
                .boxed()
            },
        )
        .await;
    }

    async fn create_team(driver: &WebDriver, port: u16) -> Result<(), WebDriverError> {
        driver
            .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .await?;

        driver
            .get(host(port, "/go/teams"))
            .await?;

        assert!(
            !driver
                .find(By::Css("[role='dialog']"))
                .await?
                .is_displayed()
                .await?
        );

        let create_btn = driver
            .find(By::Css("button[aria-label='Start creating team']"))
            .await?;
        assert_eq!(create_btn.text().await?, "Create");
        create_btn.click().await?;

        let create_dialog = driver.find(By::Css("[role='dialog']")).await?;
        create_dialog.wait_until().displayed().await?;
        assert_eq!(
            create_dialog
                .find(By::Tag("h5"))
                .await?
                .text()
                .await?,
            "Create team"
        );

        assert_create_form_is_empty(driver).await;

        create_dialog
            .find(By::Name("slug"))
            .await?
            .send_keys("slug1")
            .await?;

        create_dialog
            .find(By::Name("title"))
            .await?
            .send_keys("title1")
            .await?;

        create_dialog
            .find(By::Name("is_private"))
            .await?
            .click()
            .await?;

        create_dialog
            .find(By::Css("button[aria-label='Create team']"))
            .await?
            .click()
            .await?;

        let teams = driver
            .find_all(By::Css("[aria-label='User teams'] span"))
            .await?;
        assert_eq!(teams.last().unwrap().text().await?, "title1");

        Ok(())
    }

    async fn dialog_close_then_open(driver: &WebDriver) {
        let close = driver
            .find(By::Css("[aria-label='Close']"))
            .await
            .unwrap();
        close.click().await.unwrap();
        close.wait_until().not_displayed().await.unwrap();

        driver
            .find(By::Css("button[aria-label='Start creating team']"))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        close.wait_until().displayed().await.unwrap();
    }

    async fn assert_create_form_is_empty(driver: &WebDriver) {
        let slug = driver.find(By::Name("slug")).await.unwrap();
        assert!(slug.is_displayed().await.unwrap());
        assert_eq!(None, slug.value().await.unwrap());

        let title = driver.find(By::Name("title")).await.unwrap();
        assert!(title.is_displayed().await.unwrap());
        assert_eq!(None, title.value().await.unwrap());

        let is_private = driver.find(By::Name("is_private")).await.unwrap();
        assert!(is_private.is_displayed().await.unwrap());
        assert_eq!(
            Some("false".to_string()),
            is_private.prop("checked").await.unwrap()
        );
    }
}

mod controller {
    use super::*;

    mod delete {
        use super::*;

        #[test]
        fn without_capabilities_is_not_authorized() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", false, true, &mut conn);
            user("some_mail@mail.com", "pwd", &[], &[], &mut conn);

            let response = client.delete("/go/teams/slug1").dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .delete("/go/teams/slug1")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);
        }

        #[test]
        fn global_team_is_not_authorized() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::TeamsWrite],
                &mut conn,
            );

            client
                .delete("/go/teams/")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert!(get_team("", &mut conn).is_some());
        }

        #[test]
        fn as_admin() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", false, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::TeamsWrite],
                &mut conn,
            );

            let response = client
                .delete("/go/teams/slug1")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);
            assert!(get_team("slug1", &mut conn).is_none());
        }

        #[test]
        fn as_teamate() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", false, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                &[],
                &mut conn,
            );

            let response = client
                .delete("/go/teams/slug1")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);
            assert!(get_team("slug1", &mut conn).is_none());
        }

        #[test]
        fn as_teamate_not_accepted() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", false, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[TeamCapability::TeamsWrite], 0, false)],
                &[],
                &mut conn,
            );

            let response = client
                .delete("/go/teams/slug1")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);
        }
    }

    mod patch {
        use super::*;

        #[test]
        fn withouy_capabilities_is_not_authorized() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", false, true, &mut conn);
            user("some_mail@mail.com", "pwd", &[], &[], &mut conn);

            let response = client.delete("/go/teams/slug1").dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .patch("/go/teams/slug1")
                .json(&json!({ "title": "newTitle", "is_private": true, "is_accepted": true }))
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);
        }

        #[test]
        fn global_team_is_not_authorized() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::TeamsWrite],
                &mut conn,
            );

            client
                .patch("/go/teams/")
                .json(&json!({ "title": "newTitle", "is_private": true, "is_accepted": true }))
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(
                get_team("", &mut conn),
                Some(Team {
                    slug: "".to_string(),
                    title: "Global".to_string(),
                    is_private: false,
                    is_accepted: true
                })
            );
        }

        #[test]
        fn as_teamate() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", false, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                &[],
                &mut conn,
            );

            let response = client
                .patch("/go/teams/slug1")
                .json(&json!({ "title": "newTitle", "is_private": true }))
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);
            assert_eq!(
                get_team("slug1", &mut conn),
                Some(Team {
                    slug: "slug1".to_string(),
                    title: "newTitle".to_string(),
                    is_private: true,
                    is_accepted: true
                })
            );
        }

        #[test]
        fn accept_as_teamate_is_not_authorized() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", false, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[TeamCapability::TeamsWrite], 0, true)],
                &[],
                &mut conn,
            );

            let response = client
                .patch("/go/teams/slug1")
                .json(&json!({ "is_accepted": true }))
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);
        }

        #[test]
        fn as_admin() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", false, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::TeamsWrite],
                &mut conn,
            );

            let response = client
                .patch("/go/teams/slug1")
                .json(&json!({ "title": "newTitle", "is_private": true, "is_accepted": true }))
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);
            assert_eq!(
                get_team("slug1", &mut conn),
                Some(Team {
                    slug: "slug1".to_string(),
                    title: "newTitle".to_string(),
                    is_private: true,
                    is_accepted: true
                })
            );

            let response = client
                .patch("/go/teams/slug1")
                .json(&json!({ "title": "newTitle2", }))
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);
            assert_eq!(
                get_team("slug1", &mut conn),
                Some(Team {
                    slug: "slug1".to_string(),
                    title: "newTitle2".to_string(),
                    is_private: true,
                    is_accepted: true
                })
            );

            let response = client
                .patch("/go/teams/slug1")
                .json(&json!({ "is_private": false }))
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);
            assert_eq!(
                get_team("slug1", &mut conn),
                Some(Team {
                    slug: "slug1".to_string(),
                    title: "newTitle2".to_string(),
                    is_private: false,
                    is_accepted: true
                })
            );

            let response = client
                .patch("/go/teams/slug1")
                .json(&json!({ "is_accepted": false }))
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);
            assert_eq!(
                get_team("slug1", &mut conn),
                Some(Team {
                    slug: "slug1".to_string(),
                    title: "newTitle2".to_string(),
                    is_private: false,
                    is_accepted: false
                })
            );
        }
    }

    mod post {
        use super::*;

        #[test]
        fn already_existing_is_not_allowed() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", false, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::TeamsWrite],
                &mut conn,
            );

            client
                .post("/go/teams")
                .json(&json!({ "slug": "slug1", "title": "newTitle", "is_private": true }))
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert!(get_team("", &mut conn).is_some());
        }

        #[test]
        fn as_admin() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::TeamsWrite],
                &mut conn,
            );

            let response = client
                .post("/go/teams")
                .json(&json!({ "slug": "slug1", "title": "newTitle", "is_private": true }))
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Created);
            assert_eq!(
                get_team("slug1", &mut conn),
                Some(Team {
                    slug: "slug1".to_string(),
                    title: "newTitle".to_string(),
                    is_private: true,
                    is_accepted: true
                })
            );
        }

        #[test]
        fn as_user() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::TeamsWriteWithValidation],
                &mut conn,
            );

            let response = client
                .post("/go/teams")
                .json(&json!({ "slug": "slug1", "title": "newTitle", "is_private": true }))
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Created);
            assert_eq!(
                get_team("slug1", &mut conn),
                Some(Team {
                    slug: "slug1".to_string(),
                    title: "newTitle".to_string(),
                    is_private: true,
                    is_accepted: false
                })
            );
        }

        #[test]
        fn creator_should_be_in_team_as_admin_with_higher_rank() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug", "title", true, false, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug", &[], 0, true)],
                &[Capability::TeamsWrite],
                &mut conn,
            );

            let response = client
                .post("/go/teams")
                .json(&json!({ "slug": "slug1", "title": "newTitle", "is_private": true }))
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Created);
            assert_eq!(
                get_user_team_links("some_mail@mail.com", &mut conn),
                vec![
                    UserTeam {
                        user_mail: "some_mail@mail.com".to_string(),
                        team_slug: "slug".to_string(),
                        capabilities: vec![],
                        is_accepted: true,
                        rank: 0
                    },
                    UserTeam {
                        user_mail: "some_mail@mail.com".to_string(),
                        team_slug: "slug1".to_string(),
                        capabilities: TeamCapability::all(),
                        is_accepted: true,
                        rank: 1
                    }
                ]
            );
        }
    }
}
