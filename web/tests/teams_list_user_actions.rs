use diesel::PgConnection;
use go_web::users::{Capability, UserTeam};
use rocket::futures::FutureExt;
use rocket::http::Status;
use rocket::tokio::sync::Mutex;
use rocket::{async_test, http};
use serde_json::json;
use thirtyfour::prelude::*;

mod utils;
use go_web::guards::SESSION_COOKIE;

use utils::*;

#[async_test]
async fn join_team() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[],
                    &[Capability::UsersTeamsRead, Capability::UsersTeamsWrite],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;

                driver
                    .get(host(port, "/go/teams"))
                    .await?;

                let button = driver
                    .find(By::Css(
                        "[aria-label='Other teams'] [role='listitem'] button",
                    ))
                    .await?;
                assert_eq!(button.text().await?, "Join");
                button.click().await?;

                assert_eq!(
                    driver
                        .find(By::Css(
                            "[aria-label='User teams'] [role='listitem'] button"
                        ))
                        .await?
                        .text()
                        .await?,
                    "Leave"
                );

                driver
                    .get(host(port, "/go/teams"))
                    .await?;

                let leave = driver
                    .find(By::Css(
                        "[aria-label='User teams'] [role='listitem'] button",
                    ))
                    .await?;
                assert_eq!(leave.text().await?, "Leave");

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn leave_team() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("", &[], 0, true)],
                    &[Capability::UsersTeamsRead, Capability::UsersTeamsWrite],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;

                driver
                    .get(host(port, "/go/teams"))
                    .await?;

                let leave = driver
                    .find(By::Css(
                        "[aria-label='User teams'] [role='listitem'] button",
                    ))
                    .await?;
                assert_eq!(leave.text().await?, "Leave");
                leave.click().await?;

                assert_eq!(
                    driver
                        .find(By::Css(
                            "[aria-label='Other teams'] [role='listitem'] button"
                        ))
                        .await?
                        .text()
                        .await?,
                    "Join"
                );

                driver
                    .get(host(port, "/go/teams"))
                    .await?;

                assert_eq!(
                    driver
                        .find(By::Css(
                            "[aria-label='Other teams'] [role='listitem'] button"
                        ))
                        .await?
                        .text()
                        .await?,
                    "Join"
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn change_user_teams_rank() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                team("slug1", "team1", false, true, &mut con);
                team("slug2", "team2", false, true, &mut con);
                team("slug3", "team3", false, true, &mut con);
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[
                        ("", &[], 1, true),
                        ("slug1", &[], 2, true),
                        ("slug2", &[], 0, true),
                    ],
                    &[Capability::UsersTeamsWrite],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;

                driver
                    .get(host(port, "/go/teams"))
                    .await?;

                assert_eq!(
                    driver
                        .find(By::Css("[role='alert']"))
                        .await?
                        .text()
                        .await?,
                    "Drag and drop to prioritize team shortcuts in case of duplicates"
                );
                let teams = driver
                    .find_all(By::Css("[aria-label='User teams'] [role='listitem'] span"))
                    .await?;

                let expected_teams_sorted = vec!["team2", "Global", "team1"];
                for i in 0..expected_teams_sorted.len() {
                    assert_eq!(expected_teams_sorted[i], teams[i].text().await?);
                }

                driver
                    .find(By::Css(
                        "[aria-label='Other teams'] [role='listitem'] button",
                    ))
                    .await?
                    .click()
                    .await?;

                let teams = driver
                    .find_all(By::Css("[aria-label='User teams'] [role='listitem'] span"))
                    .await?;

                let expected_teams_sorted = vec!["team2", "Global", "team1", "team3"];
                for i in 0..expected_teams_sorted.len() {
                    assert_eq!(expected_teams_sorted[i], teams[i].text().await?);
                }

                // drag & drop testing do not work
                // let teams_rows = driver
                //     .find_all(By::Css("[aria-label='User teams'] [role='listitem']"))
                //     .await?;

                // driver
                //     .action_chain()
                //     .drag_and_drop_element(
                //         &teams_rows[2],
                //         &driver.find(By::Id("draggable")).await?,
                //     )
                //     .perform()
                //     .await?;

                // let teams = driver
                //     .find_all(By::Css("[aria-label='User teams'] [role='listitem'] span"))
                //     .await?;

                // let expected_teams_sorted = vec!["team1", "team2", "Global"];
                // for i in 0..expected_teams_sorted.len() {
                //     assert_eq!(expected_teams_sorted[i], teams[i].text().await?);
                // }

                // driver
                //     .get(host(port, "/go/teams"))
                //     .await?;

                // let teams = driver
                //     .find_all(By::Css("[aria-label='User teams'] [role='listitem'] span"))
                //     .await?;
                // for i in 0..expected_teams_sorted.len() {
                //     assert_eq!(expected_teams_sorted[i], teams[i].text().await?);
                // }

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

mod controller {
    use super::*;

    mod post {
        use super::*;

        #[test]
        fn as_unknow_user_is_not_authorized() {
            let (client, _conn) = launch_with("");

            let response = client
                .post("/go/user/teams/slug1")
                .body(json!({ "rank": 0 }).to_string())
                .dispatch();
            assert_eq!(response.status(), Status::Unauthorized);
        }

        #[test]
        fn as_user() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", false, false, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[],
                &[Capability::UsersTeamsWrite],
                &mut conn,
            );

            let response = client
                .post("/go/user/teams/slug1")
                .body(json!({ "rank": 0 }).to_string())
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();
            assert_eq!(response.status(), Status::Created);
        }
    }

    mod delete {
        use super::*;

        #[test]
        fn as_unknow_user_is_not_authorized() {
            let (client, _conn) = launch_with("");

            let response = client.delete("/go/user/teams/slug1").dispatch();
            assert_eq!(response.status(), Status::Unauthorized);
        }

        #[test]
        fn delete_user_team() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", false, false, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[("slug1", &[], 0, true)],
                &[Capability::UsersTeamsWrite],
                &mut conn,
            );

            let response = client
                .delete("/go/user/teams/slug1")
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();
            assert_eq!(response.status(), Status::Ok);
        }
    }

    mod put {
        use super::*;

        #[test]
        fn as_unknow_user_is_not_authorized() {
            let (client, _conn) = launch_with("");

            let response = client
                .put("/go/user/teams/ranks")
                .body(json!({ "": 1, "slug1": 0 }).to_string())
                .dispatch();
            assert_eq!(response.status(), Status::Unauthorized);
        }

        #[test]
        fn put_user_teams_ranks() {
            let (client, mut conn) = launch_with("some_session_id: some_mail@mail.com");
            team("slug1", "team1", false, true, &mut conn);
            user(
                "some_mail@mail.com",
                "pwd",
                &[("", &[], 0, true), ("slug1", &[], 1, true)],
                &[Capability::UsersTeamsWrite],
                &mut conn,
            );

            let response = client
                .put("/go/user/teams/ranks")
                .body(json!({ "": 1, "slug1": 0 }).to_string())
                .cookie(http::Cookie::new(SESSION_COOKIE, "some_session_id"))
                .dispatch();

            let users_teams = get_user_team_links("some_mail@mail.com", &mut conn);

            assert_eq!(
                vec![
                    UserTeam {
                        user_mail: "some_mail@mail.com".to_string(),
                        team_slug: "".to_string(),
                        capabilities: vec![],
                        is_accepted: true,
                        rank: 1
                    },
                    UserTeam {
                        user_mail: "some_mail@mail.com".to_string(),
                        team_slug: "slug1".to_string(),
                        capabilities: vec![],
                        is_accepted: true,
                        rank: 0
                    }
                ],
                users_teams
            );

            assert_eq!(response.status(), Status::Ok);
        }
    }
}
