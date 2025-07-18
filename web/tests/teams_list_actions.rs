use diesel::PgConnection;
use go_web::teams::{Team, TeamCapability};
use go_web::users::{Capability, UserTeam};
use rocket::futures::FutureExt;
use rocket::http::Status;
use rocket::tokio::sync::Mutex;
use rocket::{async_test, http};
use serde_json::json;
use thirtyfour::error::WebDriverError;
use thirtyfour::prelude::*;
use thirtyfour_testing_library_ext::{Screen, By as ByExt, TextMatch};

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
                    &[Capability::TeamsWrite, Capability::TeamsCreateWithValidation],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;

                driver
                    .get(host(port, "/go/teams"))
                    .await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                assert!(screen
                    .query(ByExt::role("button").name(TextMatch::Exact("Accept team".to_string())))
                    .await?
                    .is_none());

                screen
                    .find(ByExt::role("button").name(TextMatch::Exact("Administrate".to_string())))
                    .await?
                    .click()
                    .await?;

                // let team_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/Team list/".to_string()))).await?;
                // let teams = screen.within(team_list).find_all(ByExt::role("listitem")).await?;
                // assert!(dbg!(
                //     teams.first().unwrap().text().await?
                // )
                // .starts_with("team1"));

                let accept_btn = screen
                    .find(ByExt::role("button").name(TextMatch::Exact("Accept team".to_string())))
                    .await?;
                accept_btn.click().await?;
                assert!(screen
                    .query(ByExt::role("button").name(TextMatch::Exact("Accept team".to_string())))
                    .await?
                    .is_none());

                driver
                    .get(host(port, "/go/teams"))
                    .await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                screen
                    .find(ByExt::role("button").name(TextMatch::Exact("Administrate".to_string())))
                    .await?
                    .click()
                    .await?;

                assert!(screen
                    .query(ByExt::role("button").name(TextMatch::Exact("Accept team".to_string())))
                    .await?
                    .is_none());

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

                    let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
                    assert!(screen
                        .query(ByExt::role("button").name(TextMatch::Exact("Delete team".to_string())))
                        .await.unwrap()
                        .is_none());

                    assert!(screen
                        .query(ByExt::role("button").name(TextMatch::Exact("Administrate".to_string())))
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

        let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
        assert!(screen
            .query(ByExt::role("button").name(TextMatch::Exact("Delete team".to_string())))
            .await.unwrap()
            .is_none());

        screen
            .find(ByExt::role("button").name(TextMatch::Exact("Administrate".to_string())))
            .await
            .unwrap()
            .click()
            .await
            .unwrap();

        screen.find(ByExt::text("team1")).await.unwrap();

        let buttons = screen
            .query_all(ByExt::role("button").name(TextMatch::Exact("Delete team".to_string())))
            .await
            .unwrap();
        let delete_btn = buttons.first().unwrap();
        delete_btn.click().await.unwrap();

       let team= screen.query(ByExt::text("team1")).await.unwrap();
        assert!(team.is_none(), "Team should be deleted");

        driver
            .get(host(port, "/go/teams"))
            .await
            .unwrap();

    let team= screen.query(ByExt::text("team1")).await.unwrap();
        assert!(team.is_none(), "Team should be deleted");
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

                    let screen = Screen::build_with_testing_library(driver.clone()).await?;
                    let create_dialog = screen.find(ByExt::role("dialog")).await?;
                    let dialog_screen = screen.within(create_dialog);
                    dialog_screen
                        .find(ByExt::text("Success !"))
                        .await?;
                    let waiting_for_approval = dialog_screen
                        .query(ByExt::text("Your Admins will now have to validate your team."))
                        .await?;
                    assert!(waiting_for_approval.is_none());

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
                        &[Capability::TeamsCreateWithValidation],
                        &mut con,
                    );

                    create_team(driver, port).await?;

                    let screen = Screen::build_with_testing_library(driver.clone()).await?;
                    let create_dialog = screen.find(ByExt::role("dialog")).await?;
                    let dialog_screen = screen.within(create_dialog);
                    dialog_screen
                        .find(ByExt::text("Success !"))
                        .await?;
                    dialog_screen
                        .find(ByExt::text("Your Admins will now have to validate your team."))
                        .await?;

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

        let screen = Screen::build_with_testing_library(driver.clone()).await?;
        let dialog = screen.query(ByExt::role("dialog")).await?;
        assert!(dialog.is_none() || !dialog.unwrap().is_displayed().await?);

        let create_btn = screen
            .find(ByExt::role("button").name(TextMatch::Exact("Start creating team".to_string())))
            .await?;
        assert_eq!(create_btn.text().await?, "Create");
        create_btn.click().await?;

        let create_dialog = screen.find(ByExt::role("dialog")).await?;
        create_dialog.wait_until().displayed().await?;
        let dialog_screen = screen.within(create_dialog.clone());
        assert_eq!(
            dialog_screen
                .find(ByExt::role("heading").name(TextMatch::Exact("Create team".to_string())))
                .await?
                .text()
                .await?,
            "Create team"
        );

        assert_create_form_is_empty(driver).await;

        dialog_screen
            .find(ByExt::label_text("Slug"))
            .await?
            .send_keys("slug1")
            .await?;

        dialog_screen
            .find(ByExt::label_text("Title"))
            .await?
            .send_keys("title1")
            .await?;

        dialog_screen
            .find(ByExt::label_text("Private"))
            .await?
            .click()
            .await?;

        dialog_screen
            .find(ByExt::role("button").name(TextMatch::Exact("Create team".to_string())))
            .await?
            .click()
            .await?;

        screen.find(ByExt::text("title1")).await?;
        Ok(())
    }

    async fn assert_create_form_is_empty(driver: &WebDriver) {
        let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
        let slug = screen.find(ByExt::label_text("Slug")).await.unwrap();
        assert!(slug.is_displayed().await.unwrap());
        assert_eq!(Some("".to_string()), slug.value().await.unwrap());

        let title = screen.find(ByExt::label_text("Title")).await.unwrap();
        assert!(title.is_displayed().await.unwrap());
        assert_eq!(Some("".to_string()), title.value().await.unwrap());

        let is_private = screen.find(ByExt::label_text("Private")).await.unwrap();
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
        fn without_capabilities_is_not_authorized() {
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
        fn accept_as_partial_admin_is_not_authorized() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com
some_other_session_id: some_other_mail@mail.com");
            team("slug1", "team1", false, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::TeamsCreateWithValidation],
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
                &[Capability::TeamsCreateWithValidation],
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
