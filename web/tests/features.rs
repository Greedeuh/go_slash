use diesel::SqliteConnection;
use go_web::models::features::{Features, LoginFeature};
use rocket::futures::FutureExt;
mod utils;
use rocket::http::{Cookie, Header, Status};
use rocket::tokio::sync::Mutex;
use serde_json::json;
use serial_test::serial;
use std::default::Default;
use thirtyfour_sync::prelude::*;
use utils::*;

use go_web::guards::SESSION_COOKIE;

#[test]
#[serial]
fn features_should_list_editable_features() {
    in_browser("", |driver: &WebDriver, _con: SqliteConnection| {
        driver.get("http://localhost:8001/go/features").unwrap();

        let features = driver.find_elements(By::Css("[role='article']")).unwrap();

        assert!(!features.is_empty());

        for feature in features {
            let switch = feature.find_element(By::Css("[role='switch']")).unwrap();
            assert_eq!(
                switch.get_property("checked").unwrap(),
                Some("false".to_owned())
            );
            // switch.click().unwrap();
            // assert_eq!(
            //     switch.get_property("checked").unwrap(),
            //     Some("true".to_owned())
            // );
        }

        driver.get("http://localhost:8001/go/features").unwrap();

        // TODO re-use when having another feature
        // let features = driver
        //     .find_elements(By::Css("[role='article']"))
        //
        //     .unwrap();

        // assert!(!features.is_empty());

        // for feature in features {
        //     assert_eq!(feature.text().unwrap(), "simple");
        //     let switch = feature
        //         .find_element(By::Css("[role='switch']"))
        //
        //         .unwrap();
        //     assert_eq!(
        //         switch.get_property("checked").unwrap(),
        //         Some("true".to_owned())
        //     );
        // }
    });
}

#[test]
fn should_be_logged_in_to_manage_features() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");

    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                ..Default::default()
            },
            ..Default::default()
        },
        &conn,
    );

    assert_eq!(
        client.get("/go/features").dispatch().status(),
        Status::Unauthorized
    );
    assert_eq!(
        client
            .patch("/go/features")
            .json(&json!({ "login": null }))
            .dispatch()
            .status(),
        Status::Unauthorized
    );
}

#[test]
fn should_be_logged_in_to_manage_features_ok_with_auth() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");

    global_features(
        &Features {
            login: LoginFeature {
                simple: true,
                ..Default::default()
            },
            ..Default::default()
        },
        &conn,
    );

    assert_ne!(
        client
            .get("/go/features")
            .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .dispatch()
            .status(),
        Status::Unauthorized
    );
    assert_ne!(
        client
            .get("/go/features")
            .header(Header::new("Authorization", "some_session_id"))
            .dispatch()
            .status(),
        Status::Unauthorized
    );

    assert_ne!(
        client
            .patch("/go/features")
            .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .dispatch()
            .status(),
        Status::Unauthorized
    );
    assert_ne!(
        client
            .patch("/go/features")
            .header(Header::new("Authorization", "some_session_id"))
            .dispatch()
            .status(),
        Status::Unauthorized
    );
}
