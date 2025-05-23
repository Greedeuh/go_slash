pub use no_dead_code::*;

#[allow(dead_code)]
mod no_dead_code {
    use diesel::{prelude::*, Connection, PgConnection};
    use go_web::{models::users::Sessions, server, services::oidc::OidcService, AppConfig};
    use lazy_static::lazy_static;
    use openidconnect::{
        core::{CoreClient, CoreProviderMetadata},
        AuthUrl, ClientId, ClientSecret, IssuerUrl, JsonWebKeySetUrl, RedirectUrl, TokenUrl,
    };
    use rocket::{
        futures::{future::BoxFuture, FutureExt},
        local::blocking::Client,
        tokio::{spawn, sync::Mutex},
    };
    use std::{
        env, fmt::format, panic::{resume_unwind, AssertUnwindSafe}, time::Duration
    };
    pub use tf::*;
    use thirtyfour::{error::WebDriverError, ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};
    use uuid::Uuid;

    const PORT: u16 = 8001;
    const ADDR: &str = "127.0.0.1";
    const HOST: &str = "http://host.docker.internal";

    lazy_static! {
        static ref AVAILABLE_PORTS: Mutex<Vec<u16>> = Mutex::new((8001..9000).collect());
    }

    pub fn host(port: u16, path: &str) -> String {
        format!("{}:{}{}", HOST, port, path)
    }

    pub fn random_pg_url() -> (String, String) {
        let uuid = dbg!(format!("go_{}", Uuid::new_v4().simple()));
        (
            format!("postgres://postgres:postgres@localhost:6543/{}", uuid),
            uuid,
        )
    }

    pub fn setup_db_conn(db_url: &str, db: &str) -> PgConnection {
        let pg = PgConnection::establish("postgres://postgres:postgres@localhost:6543/postgres")
            .unwrap();
        diesel::dsl::sql::<bool>(&format!("CREATE DATABASE {};", db))
            .execute(&pg)
            .unwrap();
        PgConnection::establish(db_url).unwrap()
    }

    fn drop_db(db: &str, pg: &PgConnection) {
        diesel::dsl::sql::<bool>(&format!(
            "SELECT pg_terminate_backend(pg_stat_activity.pid)
        FROM pg_stat_activity
        WHERE pg_stat_activity.datname = '{}';",
            db
        ))
        .execute(pg)
        .unwrap();
        diesel::dsl::sql::<bool>(&format!("DROP DATABASE {};", db))
            .execute(pg)
            .unwrap();
    }

    pub fn launch_with(sessions: &str) -> (Client, PgConnection) {
        let (db_path, db) = random_pg_url();
        let db_conn = setup_db_conn(&db_path, &db);

        (
            Client::tracked(server(
                PORT,
                ADDR,
                &db_path,
                Sessions::from(sessions),
                conf(),
                true,
                true,
                google_oauth(),
            ))
            .expect("valid rocket instance"),
            db_conn,
        )
    }

    pub fn launch_with_sessions_and_mock(
        sessions: &str,
        oidc_service: OidcService,
    ) -> (Client, PgConnection) {
        let (db_path, db) = random_pg_url();
        let db_conn = setup_db_conn(&db_path, &db);

        (
            Client::tracked(server(
                PORT,
                ADDR,
                &db_path,
                Sessions::from(sessions),
                conf(),
                true,
                true,
                oidc_service,
            ))
            .expect("valid rocket instance"),
            db_conn,
        )
    }

    fn conf() -> AppConfig {
        AppConfig {
            simple_login_salt1: "salt1".to_owned(),
            simple_login_salt2: "salt2".to_owned(),
        }
    }

    fn google_oauth() -> OidcService {
        let client_id = "gclient_id".to_string();
        let client_secret = "gclient_secret".to_string();
        let hostname = "hostname".to_string();

        let provider_metadata = CoreProviderMetadata::new(
            IssuerUrl::new("http://issuer_url".to_string()).unwrap(),
            AuthUrl::new("http://auth_url/g".to_string()).unwrap(),
            JsonWebKeySetUrl::new("http://jwks_url/g".to_string()).unwrap(),
            vec![],
            vec![],
            vec![],
            openidconnect::EmptyAdditionalProviderMetadata {},
        )
        .set_token_endpoint(Some(
            TokenUrl::new("http://token_url/g".to_string()).unwrap(),
        ));
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

    pub async fn in_browser<F>(sessions: &str, f: F)
    where
        F: for<'a> FnOnce(
            &'a WebDriver,
            Mutex<PgConnection>,
            u16,
        ) -> BoxFuture<'a, Result<(), WebDriverError>>,
    {
        in_browser_with(sessions, f, true, true).await;
    }

    /// Same but launch browser
    #[deprecated(note = "Should only be used in local")]
    pub async fn in_browserr<F>(sessions: &str, f: F)
    where
        F: for<'a> FnOnce(
            &'a WebDriver,
            Mutex<PgConnection>,
            u16,
        ) -> BoxFuture<'a, Result<(), WebDriverError>>,
    {
        in_browser_with(sessions, f, false, true).await;
    }

    /// Same but launch browser and do not kill it
    #[deprecated(note = "Should only be used in local")]
    pub async fn in_browserrr<F>(sessions: &str, f: F)
    where
        F: for<'a> FnOnce(
            &'a WebDriver,
            Mutex<PgConnection>,
            u16,
        ) -> BoxFuture<'a, Result<(), WebDriverError>>,
    {
        in_browser_with(sessions, f, false, false).await;
    }

    async fn in_browser_with<'b, F>(sessions: &str, f: F, headless: bool, close_browser: bool)
    where
        F: for<'a> FnOnce(
            &'a WebDriver,
            Mutex<PgConnection>,
            u16,
        ) -> BoxFuture<'a, Result<(), WebDriverError>>,
    {
        let do_not_close_browser = close_browser;
        let do_not_close_browser = !match env::var("CLOSE_BROWSER") {
            Ok(var) => do_not_close_browser || var == "true",
            _ => do_not_close_browser,
        };

        let headless = !match env::var("HEADLESS") {
            Ok(var) => do_not_close_browser || var == "false" || !headless,
            _ => do_not_close_browser || !headless,
        };

        let (db_path, db) = random_pg_url();
        let srv_db_path = db_path.clone();
        let sessions = Sessions::from(sessions);

        let db_conn = setup_db_conn(&db_path, &db);

        let port;
        {
            port = AVAILABLE_PORTS.lock().await.remove(0);
        }

        spawn(async move {
            server(
                port,
                ADDR,
                &srv_db_path,
                sessions,
                conf(),
                true,
                true,
                google_oauth(),
            )
            .launch()
            .await
            .unwrap()
        });

        let mut caps = DesiredCapabilities::chrome();
        if headless {
            caps.set_headless().expect("Headless conf failed");
        }

        let driver = WebDriver::new("http://localhost:4444", caps)
            .await
            .expect("Driver build failed");

        let mut count = 0;
        while driver
            .get(host(port, "/go/health"))
            .await
            .is_err()
            && count < 50
        {
            count += 1;
            if count == 50 {
                log::error!("Waiting for test server timeout.",)
            }
            sleep();
        }

        let may_panic;
        {
            let db_conn = Mutex::new(db_conn);
            may_panic = AssertUnwindSafe(async { f(&driver, db_conn, port).await.unwrap() });
        }

        let maybe_panicked = may_panic.catch_unwind().await;

        {
            AVAILABLE_PORTS.lock().await.push(port);
        }

        if !do_not_close_browser {
            driver.quit().await.expect("Driver quit failed");
        }

        if let Err(panic) = maybe_panicked {
            resume_unwind(panic)
        }

        let db_conn =
            PgConnection::establish("postgres://postgres:postgres@localhost:6543/postgres")
                .unwrap();
        drop_db(&db, &db_conn);
    }

    pub fn sleep() {
        std::thread::sleep(Duration::from_millis(100));
    }

    pub mod tf {
        use go_web::guards::SESSION_COOKIE;
        use serde_json::json;
        use thirtyfour::{Cookie, WebDriver};

        pub async fn session(driver: &WebDriver, session_id: &str) {
            driver.delete_cookie(SESSION_COOKIE).await.unwrap();
            driver
                .add_cookie(Cookie::new(SESSION_COOKIE, session_id))
                .await
                .unwrap();
        }

        pub async fn refresh_with_session(driver: &WebDriver, session_id: &str) {
            session(driver, session_id).await;
            driver.refresh().await.unwrap();
        }
    }
}
