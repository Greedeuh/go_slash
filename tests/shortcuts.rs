#![feature(async_closure)]
use rocket::futures::FutureExt;
use rocket::http::ContentType;
use rocket::{async_test, http::Status};
mod helpers;
use helpers::*;
use thirtyfour::prelude::*;

#[test]
fn undefined_shortcut_return_a_404() {
    let client = launch_empty();
    let response = client.get("/myShortCut").dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn shortcut_redirect_to_target() {
    let client = launch_with("myShortCut/hop: https://thetarget.test.go.com");
    let response = client.get("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::PermanentRedirect);
    let mut location = response.headers().get("Location");

    assert_eq!(location.next(), Some("https://thetarget.test.go.com"));
    assert_eq!(location.next(), None);
}

#[async_test]
async fn undefined_shortcut_return_a_form_to_create_a_shortcut() {
    in_browser(|driver: &WebDriver| {
        async {
            // create shortcut
            driver
                .get("http://localhost:8000/newShortcut")
                .await
                .unwrap();

            let alert = driver.find_element(By::Css("[role=alert]")).await.unwrap();
            assert_eq!(
                alert.text().await.unwrap(),
                "Shortcut \"newShortcut\" does not exist yet."
            );

            let form = driver.find_element(By::Tag("form")).await.unwrap();
            let input = form
                .find_element(By::Css("input[type=text]"))
                .await
                .unwrap();
            assert_eq!(
                input.get_attribute("placeholder").await.unwrap(),
                Some("https://my-favorite-tool".to_owned())
            );
            let submit = form
                .find_element(By::Css("input[type=submit]"))
                .await
                .unwrap();
            assert_eq!(
                submit.value().await.unwrap(),
                Some("Add shortcut".to_owned())
            );

            input
                .send_keys("http://localhost:8000/looped")
                .await
                .unwrap();

            submit.click().await.unwrap();

            // assert shortcut created and working
            let alert = driver.find_element(By::Css("[role=alert]")).await.unwrap();
            assert_eq!(
                alert.text().await.unwrap(),
                "Shortcut \"newShortcut\" successfully saved !"
            );

            driver
                .get("http://localhost:8000/newShortcut")
                .await
                .unwrap();
            assert_eq!(
                driver.current_url().await.unwrap(),
                "http://localhost:8000/looped"
            );
        }
        .boxed()
    })
    .await;
}

#[async_test]
async fn create_a_shortcut_with_invalid_url() {
    in_browser(|driver: &WebDriver| {
        async {
            // create shortcut
            driver
                .get("http://localhost:8000/newShortcut/boom")
                .await
                .unwrap();

            let form = driver.find_element(By::Tag("form")).await.unwrap();
            let input = form
                .find_element(By::Css("input[type=text]"))
                .await
                .unwrap();

            let submit = form
                .find_element(By::Css("input[type=submit]"))
                .await
                .unwrap();

            input.send_keys("not_a_valid_url").await.unwrap();

            submit.click().await.unwrap();

            // assert shortcut created and working
            let alert = driver.find_element(By::Css("[role=alert]")).await.unwrap();
            // ne cause the html validator should stop us
            assert_ne!(
                alert.text().await.unwrap(),
                "Wrong format for provided URL."
            );
        }
        .boxed()
    })
    .await;
}

#[test]
fn create_a_shortcut_with_invalid_url_return_400() {
    let client = launch_with("");
    let response = client
        .post("/myShortCut/hop")
        .header(ContentType::Form)
        .body("url=not_url")
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn create_a_shortcut_return_201() {
    let client = launch_with("");
    let response = client
        .post("/myShortCut/hop")
        .header(ContentType::Form)
        .body("url=http%3A%2F%2Flocalhost%3A11")
        .dispatch();

    assert_eq!(response.status(), Status::Created);
}

#[test]
fn replace_a_shortcut_return_201() {
    let client = launch_with("/myShortCut/hop: http://azdazd.dz");
    let response = client
        .post("/myShortCut/hop")
        .header(ContentType::Form)
        .body("url=http%3A%2F%2Flocalhost%3A11")
        .dispatch();

    assert_eq!(response.status(), Status::Created);
}

#[test]
fn delete_a_shortcut_return_200() {
    let client = launch_with("/myShortCut/hop: http://azdazd.dz");
    let response = client.delete("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[async_test]
async fn delete_a_shortcut() {
    in_browser(|driver: &WebDriver| {
        async {
            // create shortcut
            driver
                .get("http://localhost:8000/newShortcut/boom")
                .await
                .unwrap();

            let form = driver.find_element(By::Tag("form")).await.unwrap();

            let delete_btn = form
                .find_element(By::Css("input[role=deletion]"))
                .await
                .unwrap();

            assert_eq!(
                delete_btn.value().await.unwrap(),
                Some("Delete shortcut".to_owned())
            );

            delete_btn.click().await.unwrap();

            // assert shortcut created and working
            let alert = driver.find_element(By::Css("[role=alert]")).await.unwrap();
            // ne cause the html validator should stop us
            assert_ne!(
                alert.text().await.unwrap(),
                "Shortcut \"newShortcut\" successfully deleted !"
            );
        }
        .boxed()
    })
    .await;
}
