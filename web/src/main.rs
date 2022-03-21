#[macro_use]
extern crate rocket;
use std::env;

use go_web::{models::users::Sessions, server, AppConfig};

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().unwrap();

    let db_url = env::var("DATABASE_URL").expect("Missing DATABASE_URL env var");

    let port = env::var("PORT");
    let port = match port {
        Ok(port) => port.parse().unwrap(),
        _ => 8000,
    };

    let addr = env::var("ADDR");
    let addr = match addr {
        Ok(addr) => addr,
        _ => "127.0.0.1".to_owned(),
    };

    let simple_login_salt1 = env::var("SALT1").expect("expect env var SALT1");
    let simple_login_salt2 = env::var("SALT2").expect("expect env var SALT2");

    server(
        port,
        &addr,
        &db_url,
        Sessions::default(),
        AppConfig {
            simple_login_salt1,
            simple_login_salt2,
        },
    )
}
