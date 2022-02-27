use rocket::http::Status;

mod helpers;
use helpers::*;

#[test]
fn server_is_running() {
    let client = launch_empty();
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::Ok);
}
