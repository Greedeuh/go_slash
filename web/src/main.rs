#![feature(let_chains)]
#[macro_use]
extern crate rocket;
use log::warn;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    reqwest, ClientId, ClientSecret, IssuerUrl, RedirectUrl,
};
use std::env;

use go_web::{models::users::Sessions, server, services::oidc::OidcService, AppConfig};

#[launch]
async fn rocket() -> _ {
    if let Err(err) = dotenv::dotenv() {
        warn!("Dot env setup failed: {:?}", err)
    };

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
        google_oidc_client().await,
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
        // .chain(fern::log_file("go.log").unwrap()) TO DO see what to do with that
        .apply()
        .unwrap();
}

async fn google_oidc_client() -> OidcService {
    let client_id =
        env::var("OAUTH_GOOGLE_CLIENT_ID").expect("expect env var OAUTH_GOOGLE_CLIENT_ID");
    let client_secret =
        env::var("OAUTH_GOOGLE_CLIENT_SECRET").expect("expect env var OAUTH_GOOGLE_CLIENT_SECRET");
    let hostname = env::var("HOSTNAME").expect("expect env var SALT2");

    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new("https://accounts.google.com".to_string()).unwrap(),
        reqwest::async_http_client,
    )
    .await
    .unwrap();
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
    )
    .set_redirect_uri(
        RedirectUrl::new(format!("http://{}/go/login/redirect/google", hostname)).unwrap(),
    );

    OidcService::new(client)
}
