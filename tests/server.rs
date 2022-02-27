use rocket::http::Status;
use rocket::local::blocking::Client;

use go::server;

#[test]
fn server_is_running() {
    let client = Client::tracked(server()).expect("valid rocket instance");
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::Ok);
}
