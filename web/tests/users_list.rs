use diesel::PgConnection;
use go_web::users::Capability;
use rocket::futures::FutureExt;
use rocket::http::Status;
use rocket::tokio::sync::Mutex;
use rocket::{async_test, http};
use thirtyfour::prelude::*;

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

                assert!(driver
                    .find(By::Css("a [href='/go/users']"))
                    .await
                    .is_err());

                let endpoints = vec!["", "go/teams", "go/features", "azdaz"];

                for endpoint in endpoints {
                    driver
                        .get(format!("http://host.docker.internal:{}/{}", port, dbg!(endpoint)))
                        .await?;

                    assert_eq!(
                        driver
                            .find(By::Css("[href='/go/users']"))
                            .await?
                            .text()
                            .await?,
                        "users"
                    );
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

                let user = driver
                    .find_all(By::Css("[role='listitem']"))
                    .await?
                    .first()
                    .unwrap()
                    .clone();

                let expeted_capabilities = [("false", Capability::Features),
                    ("false", Capability::TeamsCreateWithValidation),
                    ("true", Capability::TeamsWrite),
                    ("false", Capability::UsersAdmin),
                    ("false", Capability::UsersTeamsRead),
                    ("false", Capability::UsersTeamsWrite)];

                user.click().await?;

                let switchs = user.find_all(By::Css("[role='switch']")).await?;
                let switchs_label = user.find_all(By::Tag("label")).await?;
                for i in 0..expeted_capabilities.len() {
                    let (checked, label) = expeted_capabilities[i];
                    switchs_label[i].wait_until().displayed().await?;
                    assert_eq!(switchs_label[i].text().await.unwrap(), label.to_string());
                    assert_eq!(
                        switchs[i].prop("checked").await?.unwrap(),
                        checked.to_string()
                    );
                }

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

                let user = driver
                    .find_all(By::Css("[role='listitem']"))
                    .await?
                    .first()
                    .unwrap()
                    .clone();

                user.click().await?;

                let switchs = user.find_all(By::Css("[role='switch']")).await?;
                let switch = switchs.first().unwrap();
                let switchs_label = user.find_all(By::Tag("label")).await?;
                let switch_label = switchs_label.first().unwrap();

                switch_label.wait_until().displayed().await?;
                assert_eq!(
                    switch_label.text().await.unwrap(),
                    Capability::Features.to_string()
                );
                assert_eq!(switch.prop("checked").await?.unwrap(), "false");

                switch.click().await?;
                assert_eq!(switch.prop("checked").await?.unwrap(), "true");

                driver
                    .get(host(port, "/go/users"))
                    .await?;

                driver
                    .find_all(By::Css("[role='listitem']"))
                    .await?
                    .first()
                    .unwrap()
                    .click()
                    .await?;

                let switchs = driver.find_all(By::Css("[role='switch']")).await?;
                let switch = &switchs[0];

                switch.wait_until().displayed().await?;
                assert_eq!(switch.prop("checked").await?.unwrap(), "true");

                switch.click().await?;
                assert_eq!(switch.prop("checked").await?.unwrap(), "false");

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

async fn assert_users(driver: &WebDriver, expected_users: Vec<&str>) {
    let users = driver
        .find_all(By::Css("[role='listitem'] h2"))
        .await
        .unwrap();
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
