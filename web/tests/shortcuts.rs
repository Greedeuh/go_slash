use rocket::http::ContentType;
use rocket::http::Status;
mod helpers;
use helpers::*;
use serde_json::json;
use serde_json::Value;

#[test]
fn undefined_shortcut_return_a_404() {
    let client = launch_empty();
    let response = client.get("/myShortCut").dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn shortcut_redirect_to_target() {
    let client = launch_with("myShortCut/hop: https://thetarget.test.go.com", "");
    let response = client.get("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::PermanentRedirect);
    let mut location = response.headers().get("Location");

    assert_eq!(location.next(), Some("https://thetarget.test.go.com"));
    assert_eq!(location.next(), None);
}

#[test]
fn create_a_shortcut_with_invalid_url_return_400() {
    let client = launch_with("", "");
    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .body(r#"{"url": "not_url"}"#)
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(
        response.into_string().unwrap().parse::<Value>().unwrap(),
        json!({"error": "Wrong URL format."})
    );
}

#[test]
fn create_a_shortcut_return_200() {
    let client = launch_with("", "");
    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .body(r#"{"url": "http://localhost"}"#)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn replace_a_shortcut_return_200() {
    let client = launch_with("/myShortCut/hop: http://azdazd.dz", "");
    let response = client
        .put("/myShortCut/hop")
        .header(ContentType::JSON)
        .body(r#"{"url": "http://localhost"}"#)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn delete_a_shortcut_return_200() {
    let client = launch_with("/myShortCut/hop: http://azdazd.dz", "");
    let response = client.delete("/myShortCut/hop").dispatch();

    assert_eq!(response.status(), Status::Ok);
}
