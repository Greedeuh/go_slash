#[macro_use]
extern crate diesel_migrations;

use diesel::SqliteConnection;
use go_web::controllers::users::LoginSuccessfull;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
mod utils;
use rocket::http::Status;
use serde_json::json;
use thirtyfour::prelude::*;
use utils::*;
use uuid::Uuid;

#[test]
fn simple_login_is_behind_a_feature_switch() {
    let (client, _conn) = launch_with("", "");
    let response = client.get("/go/login").dispatch();

    assert_eq!(response.status(), Status::Conflict);

    let response = client
        .post("/go/login")
        .body(json!({ "mail": "some_mail", "pwd": "some_pwd" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);
}

#[test]
fn simple_login_feature_switch() {
    let (client, _conn) = launch_with("", "");
    let response = client.get("/go/login").dispatch();

    assert_eq!(response.status(), Status::Conflict);
}

#[test]
fn post_simple_login_token() {
    let (client, conn) = launch_with(
        "---
    login:
      simple: true
      read_private: false
      write_private: false
    ",
        "",
    );
    user(
        "some_mail@mail.go",
        "b112aa82a7aafb32aea966cafd2f6bb2562c34d2f08bb1dee9fab4b2b223ea20",
        &conn,
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
fn post_simple_login_wrong_credentials() {
    let (client, conn) = launch_with(
        "---
    login:
      simple: true
      read_private: false
      write_private: false
    ",
        "",
    );
    user(
        "some_mail@mail.go",
        "b112aa82a7aafb32aea966cafd2f6bb2562c34d2f08bb1dee9fab4b2b223ea20",
        &conn,
    );

    let response = client
        .post("/go/login")
        .body(json!({ "mail": "some_mail@mail.go", "pwd": "wrong_pwd" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);

    let response = client
        .post("/go/login")
        .body(json!({ "mail": "wrong_mail@mail.go", "pwd": "some_pwd" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn post_simple_login_not_a_mail() {
    let (client, conn) = launch_with(
        "---
    login:
      simple: true
      read_private: false
      write_private: false
    ",
        "",
    );
    user(
        "some_mail@mail.go",
        "b112aa82a7aafb32aea966cafd2f6bb2562c34d2f08bb1dee9fab4b2b223ea20",
        &conn,
    );

    let response = client
        .post("/go/login")
        .body(json!({ "mail": "not_mail", "pwd": "wrong_pwd" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[async_test]
async fn simple_login() {
    in_browser(
        "---
    login:
      simple: true
      read_private: false
      write_private: false
    ",
        "",
        |driver: &WebDriver, con: Mutex<SqliteConnection>| {
            async move {
                let con = con.lock().await;
                user(
                    "some_mail@mail.go",
                    "4a4498acaf82759d929a7571b5bcea425c9275854d963e49333bf8056c673f60",
                    &con,
                );

                driver
                    .get("http://localhost:8001/go/login?from=allo")
                    .await
                    .unwrap();

                driver
                    .find_element(By::Css("[type='email']"))
                    .await
                    .unwrap()
                    .send_keys("some_mail@mail.go")
                    .await
                    .unwrap();
                driver
                    .find_element(By::Css("[type='password']"))
                    .await
                    .unwrap()
                    .send_keys("wrong_pwd")
                    .await
                    .unwrap();
                driver
                    .find_element(By::Css("[type='submit']"))
                    .await
                    .unwrap()
                    .click()
                    .await
                    .unwrap();

                sleep();

                assert_eq!(
                    driver
                        .find_element(By::Css("[role='alert']"))
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap(),
                    "Wrong credentials :/ !"
                );

                driver
                    .get("http://localhost:8001/go/login?from=/allo")
                    .await
                    .unwrap();

                driver
                    .find_element(By::Css("[type='email']"))
                    .await
                    .unwrap()
                    .send_keys("some_mail@mail.go")
                    .await
                    .unwrap();
                driver
                    .find_element(By::Css("[type='password']"))
                    .await
                    .unwrap()
                    .send_keys("some_pwd")
                    .await
                    .unwrap();
                driver
                    .find_element(By::Css("[type='submit']"))
                    .await
                    .unwrap()
                    .click()
                    .await
                    .unwrap();

                assert_eq!(
                    driver
                        .find_element(By::Css("[role='alert']"))
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap(),
                    "Login success !"
                );

                std::thread::sleep(std::time::Duration::from_millis(400));

                assert_eq!(
                    driver.current_url().await.unwrap(),
                    "http://localhost:8001/allo"
                );
                let login_link = driver.find_element(By::Css(".navbar-text")).await.unwrap();
                assert_eq!(login_link.text().await.unwrap(), "some_mail@mail.go");

                driver
                    .get("http://localhost:8001/another?no_redirect")
                    .await
                    .unwrap();
                let login_link = driver.find_element(By::Css(".navbar-text")).await.unwrap();
                assert_eq!(login_link.text().await.unwrap(), "some_mail@mail.go");

                driver.get("http://localhost:8001").await.unwrap();
                let login_link = driver.find_element(By::Css(".navbar-text")).await.unwrap();
                assert_eq!(login_link.text().await.unwrap(), "some_mail@mail.go");
            }
            .boxed()
        },
    )
    .await;
}
