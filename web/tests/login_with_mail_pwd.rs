use diesel::PgConnection;
use go_web::login::LoginSuccessfull;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
mod utils;
use rocket::http::Status;
use serde_json::json;
use thirtyfour::prelude::*;
use utils::*;
use uuid::Uuid;

#[async_test]
async fn as_user() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut conn = con.lock().await;
                user(
                    "some_mail@mail.go",
                    "4a4498acaf82759d929a7571b5bcea425c9275854d963e49333bf8056c673f60",
                    &[],
                    &[],
                    &mut conn,
                );

                driver
                    .get(host(port, "/go/login"))
                    .await?;

                driver
                    .find(By::Css("[type='email']"))
                    .await?
                    .send_keys("some_mail@mail.go")
                    .await?;
                driver
                    .find(By::Css("[type='password']"))
                    .await?
                    .send_keys("some_pwd")
                    .await?;
                driver
                    .find(By::Css("[type='submit']"))
                    .await?
                    .click()
                    .await?;

                assert_eq!(
                    driver
                        .find(By::Css("[role='alert']"))
                        .await?
                        .text()
                        .await?,
                    "Login success !"
                );

                std::thread::sleep(std::time::Duration::from_millis(500));

                assert_eq!(
                    driver.current_url().await?.to_string(),
                    host(port, "/")
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn with_from_query_param_redirect_to_it() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut conn = con.lock().await;
                user(
                    "some_mail@mail.go",
                    "4a4498acaf82759d929a7571b5bcea425c9275854d963e49333bf8056c673f60",
                    &[],
                    &[],
                    &mut conn,
                );

                driver
                    .get(host(port, "/go/login?from=/allo"))
                    .await?;

                driver
                    .find(By::Css("[type='email']"))
                    .await?
                    .send_keys("some_mail@mail.go")
                    .await?;
                driver
                    .find(By::Css("[type='password']"))
                    .await?
                    .send_keys("some_pwd")
                    .await?;
                driver
                    .find(By::Css("[type='submit']"))
                    .await?
                    .click()
                    .await?;

                assert_eq!(
                    driver
                        .find(By::Css("[role='alert']"))
                        .await?
                        .text()
                        .await?,
                    "Login success !"
                );

                std::thread::sleep(std::time::Duration::from_millis(500));

                assert_eq!(
                    driver.current_url().await?.to_string(),
                    host(port, "/allo")
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn with_wrong_credentials_show_an_error() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut conn = con.lock().await;
                user(
                    "some_mail@mail.go",
                    "4a4498acaf82759d929a7571b5bcea425c9275854d963e49333bf8056c673f60",
                    &[],
                    &[],
                    &mut conn,
                );

                driver
                    .get(host(port, "/go/login?from=allo"))
                    .await?;

                driver
                    .find(By::Css("[type='email']"))
                    .await?
                    .send_keys("some_mail@mail.go")
                    .await?;
                driver
                    .find(By::Css("[type='password']"))
                    .await?
                    .send_keys("wrong_pwd")
                    .await?;
                driver
                    .find(By::Css("[type='submit']"))
                    .await?
                    .click()
                    .await?;

                assert_eq!(
                    driver
                        .find(By::Css("[role='alert']"))
                        .await?
                        .text()
                        .await?,
                    "Wrong credentials :/ !"
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

mod controller {
    use super::*;

    #[test]
    fn as_user() {
        let (client, mut conn) = launch_with("");
        user(
            "some_mail@mail.go",
            "b112aa82a7aafb32aea966cafd2f6bb2562c34d2f08bb1dee9fab4b2b223ea20",
            &[],
            &[],
            &mut conn,
        );

        let response = client
            .post("/go/login")
            .body(json!({ "mail": "some_mail@mail.go", "pwd": "some_pwd" }).to_string())
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response
            .into_json::<LoginSuccessfull>()
            .unwrap()
            .token
            .parse::<Uuid>()
            .is_ok());
    }

    #[test]
    fn as_user_with_wrong_pwd_is_not_allowed() {
        let (client, mut conn) = launch_with("");
        user(
            "some_mail@mail.go",
            "b112aa82a7aafb32aea966cafd2f6bb2562c34d2f08bb1dee9fab4b2b223ea20",
            &[],
            &[],
            &mut conn,
        );

        let response = client
            .post("/go/login")
            .body(json!({ "mail": "some_mail@mail.go", "pwd": "wrong_pwd" }).to_string())
            .dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[test]
    fn as_user_with_wrong_mail_is_not_allowed() {
        let (client, mut conn) = launch_with("");
        user(
            "some_mail@mail.go",
            "b112aa82a7aafb32aea966cafd2f6bb2562c34d2f08bb1dee9fab4b2b223ea20",
            &[],
            &[],
            &mut conn,
        );

        let response = client
            .post("/go/login")
            .body(json!({ "mail": "wrong_mail@mail.go", "pwd": "some_pwd" }).to_string())
            .dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[test]
    fn with_bad_mail_format_is_not_allowed() {
        let (client, mut conn) = launch_with("");
        user(
            "some_mail@mail.go",
            "b112aa82a7aafb32aea966cafd2f6bb2562c34d2f08bb1dee9fab4b2b223ea20",
            &[],
            &[],
            &mut conn,
        );

        let response = client
            .post("/go/login")
            .body(json!({ "mail": "not_mail", "pwd": "wrong_pwd" }).to_string())
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }
}
