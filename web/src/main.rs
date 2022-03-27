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

    let run_migrations = env::var("RUN_MIGRATIONS");
    let run_migrations = matches!(run_migrations, Ok(run_migrations) if run_migrations == "true");

    logger();

    server(
        port,
        &addr,
        &db_url,
        Sessions::default(),
        AppConfig {
            simple_login_salt1,
            simple_login_salt2,
        },
        run_migrations,
        false,
    )
}

fn logger() {
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("go.log").unwrap())
        .apply()
        .unwrap();
}
