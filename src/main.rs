#[macro_use]
extern crate rocket;
use go::server;

#[launch]
fn run() -> _ {
    server()
}
