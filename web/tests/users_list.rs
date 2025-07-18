use diesel::PgConnection;
use go_web::users::Capability;
use rocket::futures::FutureExt;
use rocket::http::Status;
use rocket::tokio::sync::Mutex;
use rocket::{async_test, http};
use thirtyfour::prelude::*;
use thirtyfour_testing_library_ext::{Screen, By as ByExt, TextMatch};

mod utils;
use go_web::guards::SESSION_COOKIE;

use utils::*;

#[async_test]
async fn link_are_shown_on_other_pages() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                user("some_mail@mail.com", "pwd", &[], &Capability::all(), &mut con);

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;

                driver.get(host(port, "")).await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                screen
                    .find(ByExt::role("link").name(TextMatch::Exact("users".to_string())))
                    .await?;

                let endpoints = vec!["", "go/teams", "go/features", "azdaz"];

                for endpoint in endpoints {
                    driver
                        .get(format!("http://host.docker.internal:{}/{}", port, dbg!(endpoint)))
                        .await?;

                    let screen = Screen::build_with_testing_library(driver.clone()).await?;
                    let nav = screen.find(ByExt::role("navigation")).await?;
                    let nav_screen = screen.within(nav);
                    let link = nav_screen
                        .find(ByExt::text("users"))
                        .await?;
                    assert!(link.attr("href").await?.unwrap().ends_with("/go/users"));
                }

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn as_admin_i_can_see_the_list() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[],
                    &[Capability::UsersAdmin],
                    &mut con,
                );
                user(
                    "another_mail@mail.com",
                    "pwd",
                    &[],
                    &[Capability::TeamsWrite],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;

                driver
                    .get(host(port, "/go/users"))
                    .await?;

                assert_users(driver, vec!["another_mail@mail.com", "some_mail@mail.com"]).await;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                let user_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/User list/".to_string()))).await?;
                let users = screen.within(user_list).find_all(ByExt::role("listitem")).await?;
                let user = users.first().unwrap().clone();


                user.click().await?;

                let user_screen = screen.within(user.clone());
                
                let features_switch = user_screen.find(ByExt::label_text(&Capability::Features.to_string())).await?;
                assert_eq!(features_switch.prop("checked").await?.unwrap(), "false");

                let teams_create_switch = user_screen.find(ByExt::label_text(&Capability::TeamsCreateWithValidation.to_string())).await?;
                assert_eq!(teams_create_switch.prop("checked").await?.unwrap(), "false");

                let teams_write_switch = user_screen.find(ByExt::label_text(&Capability::TeamsWrite.to_string())).await?;
                assert_eq!(teams_write_switch.prop("checked").await?.unwrap(), "true");

                let users_admin_switch = user_screen.find(ByExt::label_text(&Capability::UsersAdmin.to_string())).await?;
                assert_eq!(users_admin_switch.prop("checked").await?.unwrap(), "false");

                let users_teams_read_switch = user_screen.find(ByExt::label_text(&Capability::UsersTeamsRead.to_string())).await?;
                assert_eq!(users_teams_read_switch.prop("checked").await?.unwrap(), "false");

                let users_teams_write_switch = user_screen.find(ByExt::label_text(&Capability::UsersTeamsWrite.to_string())).await?;
                assert_eq!(users_teams_write_switch.prop("checked").await?.unwrap(), "false");

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn as_admin_i_can_change_users_capabilities() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[],
                    &[Capability::UsersAdmin],
                    &mut con,
                );
                user("another_mail@mail.com", "pwd", &[], &[], &mut con);

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;

                driver
                    .get(host(port, "/go/users"))
                    .await?;

                assert_users(driver, vec!["another_mail@mail.com", "some_mail@mail.com"]).await;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                let user_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/User list/".to_string()))).await?;
                let users = screen.within(user_list).find_all(ByExt::role("listitem")).await?;
                let user = users.first().unwrap().clone();

                user.click().await?;

                let user_screen = screen.within(user.clone());
                let features_switch = user_screen.find(ByExt::label_text(&Capability::Features.to_string())).await?;
                assert_eq!(features_switch.prop("checked").await?.unwrap(), "false");

                features_switch.click().await?;
                assert_eq!(features_switch.prop("checked").await?.unwrap(), "true");

                driver
                    .get(host(port, "/go/users"))
                    .await?;

                let screen = Screen::build_with_testing_library(driver.clone()).await?;
                let user_list = screen.find(ByExt::role("list").name(TextMatch::Regex("/User list/".to_string()))).await?;
                let users = screen.within(user_list).find_all(ByExt::role("listitem")).await?;
                let user = users.first().unwrap();
                user.click().await?;

                let user_screen = screen.within(user.clone());
                let features_switch = user_screen.find(ByExt::label_text(&Capability::Features.to_string())).await?;
                assert_eq!(features_switch.prop("checked").await?.unwrap(), "true");

                features_switch.click().await?;
                assert_eq!(features_switch.prop("checked").await?.unwrap(), "false");

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

async fn assert_users(driver: &WebDriver, expected_users: Vec<&str>) {
    let screen = Screen::build_with_testing_library(driver.clone()).await.unwrap();
    let users = screen.find_all(ByExt::role("heading")).await.unwrap();
    for i in 0..expected_users.len() {
        assert_eq!(users[i].text().await.unwrap(), expected_users[i]);
    }
}

mod controller {
    use super::*;

    #[test]
    fn with_unknow_user_is_not_unauthorized() {
        let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
        user("some_mail@mail.com", "pwd", &[], &[], &mut conn);

        let response = client
            .put("/go/users/some_mail@mail.com/capabilities/Features")
            .dispatch();

        assert_eq!(response.status(), Status::Unauthorized);

        let response = client
            .put("/go/users/some_mail@mail.com/capabilities/Features")
            .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
            .dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
    }

    mod put {
        use super::*;
        #[test]
        fn as_unknown_user_is_not_unauthorized() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            user("some_mail@mail.com", "pwd", &[], &[], &mut conn);

            let response = client
                .put("/go/users/some_mail@mail.com/capabilities/Features")
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .put("/go/users/some_mail@mail.com/capabilities/Features")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);
        }

        #[test]
        fn as_user() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::UsersAdmin],
                &mut conn,
            );

            let response = client
                .put("/go/users/some_mail@mail.com/capabilities/Features")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);

            let user = get_user("some_mail@mail.com", &mut conn).unwrap();
            assert_eq!(
                user.capabilities,
                &[Capability::UsersAdmin, Capability::Features]
            )
        }
    }

    mod delete {
        use super::*;
        #[test]
        fn as_unknown_user_is_not_unauthorized() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            user("some_mail@mail.com", "pwd", &[], &[], &mut conn);

            let response = client
                .delete("/go/users/some_mail@mail.com/capabilities/Features")
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);

            let response = client
                .delete("/go/users/some_mail@mail.com/capabilities/Features")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Unauthorized);
        }

        #[test]
        fn as_user() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::UsersAdmin, Capability::Features],
                &mut conn,
            );

            let response = client
                .delete("/go/users/some_mail@mail.com/capabilities/Features")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            assert_eq!(response.status(), Status::Ok);

            let user = get_user("some_mail@mail.com", &mut conn).unwrap();
            assert_eq!(user.capabilities, &[Capability::UsersAdmin,])
        }
    }
}
