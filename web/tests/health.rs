#[macro_use]
extern crate diesel_migrations;

mod utils;
use rocket::http::Status;
use utils::*;

#[test]
fn health_check() {
    let (client, _conn) = launch_with("", "", "");
    let response = client.get("/go/health").dispatch();
    assert_eq!(response.status(), Status::Ok);
}
