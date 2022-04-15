use diesel::{Connection, SqliteConnection};
use go_web::{models::users::Sessions, server, AppConfig};
use rocket::{
    futures::{future::BoxFuture, FutureExt},
    local::blocking::Client,
    tokio::{spawn, sync::Mutex},
};
use std::{
    env,
    fs::{create_dir, write},
    panic::{resume_unwind, AssertUnwindSafe},
    time::Duration,
};
use thirtyfour::{error::WebDriverError, DesiredCapabilities, WebDriver};
use uuid::Uuid;

const PORT: u16 = 8001;
const ADDR: &str = "127.0.0.1";

#[allow(unused_must_use)]
fn gen_file_path(content: &str) -> String {
    create_dir("test_dir");

    let path = format!("test_dir/filename_{}.yml", Uuid::new_v4());
    if !content.is_empty() {
        write(&path, content).unwrap();
    }
    path
}

#[allow(dead_code)]
pub fn launch_empty() -> Client {
    let db_path = gen_file_path("");

    Client::tracked(server(
        PORT,
        ADDR,
        &db_path,
        Sessions::default(),
        conf(),
        true,
        true,
    ))
    .expect("valid rocket instance")
}

#[allow(dead_code)]
pub fn launch_with(sessions: &str) -> (Client, SqliteConnection) {
    let db_path = gen_file_path("");
    let db_conn = SqliteConnection::establish(&db_path).unwrap();

    (
        Client::tracked(server(
            PORT,
            ADDR,
            &db_path,
            Sessions::from(sessions),
            conf(),
            true,
            true,
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

#[allow(dead_code)]
pub async fn in_browser<F>(sessions: &str, f: F)
where
    F: for<'a> FnOnce(
        &'a WebDriver,
        Mutex<SqliteConnection>,
    ) -> BoxFuture<'a, Result<(), WebDriverError>>,
{
    in_browser_with(sessions, f, true, true).await;
}

#[allow(dead_code)]
/// Same but launch browser
#[deprecated(note = "Should only be used in local")]
pub async fn in_browserr<F>(sessions: &str, f: F)
where
    F: for<'a> FnOnce(
        &'a WebDriver,
        Mutex<SqliteConnection>,
    ) -> BoxFuture<'a, Result<(), WebDriverError>>,
{
    in_browser_with(sessions, f, false, true).await;
}

#[allow(dead_code)]
/// Same but launch browser and do not kill it
#[deprecated(note = "Should only be used in local")]
pub async fn in_browserrr<F>(sessions: &str, f: F)
where
    F: for<'a> FnOnce(
        &'a WebDriver,
        Mutex<SqliteConnection>,
    ) -> BoxFuture<'a, Result<(), WebDriverError>>,
{
    in_browser_with(sessions, f, false, false).await;
}

async fn in_browser_with<'b, F>(sessions: &str, f: F, headless: bool, close_browser: bool)
where
    F: for<'a> FnOnce(
        &'a WebDriver,
        Mutex<SqliteConnection>,
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

    let db_path = gen_file_path("");
    let srv_db_path = db_path.clone();
    let sessions = Sessions::from(sessions);

    let db_conn = SqliteConnection::establish(&db_path).unwrap();

    spawn(async move {
        server(PORT, ADDR, &srv_db_path, sessions, conf(), true, true)
            .launch()
            .await
            .unwrap()
    });

    let mut caps = DesiredCapabilities::chrome();
    if headless {
        caps.set_headless().expect("Headless conf failed");
    }

    let driver = WebDriver::new("http://localhost:4444", &caps)
        .await
        .expect("Driver build failed");

    let mut count = 0;
    while driver.get("http://localhost:8001/go/health").await.is_err() && count < 50 {
        count += 1;
        if count == 50 {
            log::error!("Waiting for test server timeout.",)
        }
        sleep();
    }

    let db_conn = Mutex::new(db_conn);
    let may_panic = AssertUnwindSafe(async { f(&driver, db_conn).await.unwrap() });

    let maybe_panicked = may_panic.catch_unwind().await;

    if !do_not_close_browser {
        driver.quit().await.expect("Driver quit failed");
    }

    if let Err(panic) = maybe_panicked {
        resume_unwind(panic)
    }
}

#[allow(dead_code)]
pub fn sleep() {
    std::thread::sleep(Duration::from_millis(100));
}
