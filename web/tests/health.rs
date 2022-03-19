mod helpers;
use helpers::*;
use rocket::http::Status;

#[test]
fn health_check() {
    let client = launch_with("", "", "", "");
    let response = client.get("/go/health").dispatch();
    assert_eq!(response.status(), Status::Ok);
}
