#![feature(async_closure)]
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
        Box::pin(async {
            driver.get("http://localhost:8000/newShortcut").await?;

            let p = driver.find_element(By::Tag("p")).await?;
            assert_eq!(
                p.text().await?,
                "Shortcut \"newShortcut\" does not exist yet."
            );

            let form = driver.find_element(By::Tag("form")).await?;
            let input = form.find_element(By::Css("input[type=text]")).await?;
            assert_eq!(
                input.get_attribute("placeholder").await?,
                Some("https://myFavoriteTool.com".to_owned())
            );
            let btn = form.find_element(By::Tag("form")).await?;
            assert_eq!(btn.text().await?, "Add short cut");

            Ok(())
        })
    })
    .await;
}
