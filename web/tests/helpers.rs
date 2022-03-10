use go_web::{server, Entries, GlobalFeatures};
use rocket::{
    futures::{future::BoxFuture, FutureExt},
    local::blocking::Client,
    tokio::spawn,
};
use std::{
    env,
    fs::{create_dir, remove_dir_all, write},
    panic::{resume_unwind, AssertUnwindSafe},
    time::Duration,
};
use thirtyfour::{DesiredCapabilities, WebDriver};
use uuid::Uuid;

const PORT: u16 = 8001;

fn gen_file_path(content: &str) -> String {
    if let Err(e) = remove_dir_all("test_dir") {
        println!("{:?}", e);
    };

    create_dir("test_dir").unwrap();

    let path = format!("test_dir/filename_{}.yml", Uuid::new_v4());
    if !content.is_empty() {
        write(&path, content).unwrap();
    }
    path
}

#[allow(dead_code)]
pub fn launch_empty() -> Client {
    Client::tracked(server(
        PORT,
        Entries::from_path(&gen_file_path("")),
        GlobalFeatures::from_path(&gen_file_path("")),
    ))
    .expect("valid rocket instance")
}

#[allow(dead_code)]
pub fn launch_with(shortcuts: &str, features: &str) -> Client {
    Client::tracked(server(
        PORT,
        Entries::from_path(&gen_file_path(shortcuts)),
        GlobalFeatures::from_path(&gen_file_path(features)),
    ))
    .expect("valid rocket instance")
}

#[allow(dead_code)]
pub fn entries(shortcuts: &str) -> Entries {
    Entries::from_path(&gen_file_path(shortcuts))
}

#[allow(dead_code)]
pub async fn in_browser<F>(shortcuts: &str, features: &str, f: F)
where
    F: for<'a> FnOnce(&'a WebDriver) -> BoxFuture<'a, ()>,
{
    in_browser_with(shortcuts, features, f, true, true).await;
}

#[allow(dead_code)]
/// Same but launch browser
#[deprecated(note = "Should only be used in local")]
pub async fn in_browserr<F>(shortcuts: &str, features: &str, f: F)
where
    F: for<'a> FnOnce(&'a WebDriver) -> BoxFuture<'a, ()>,
{
    in_browser_with(shortcuts, features, f, false, true).await;
}

#[allow(dead_code)]
/// Same but launch browser and do not kill it
#[deprecated(note = "Should only be used in local")]
pub async fn in_browserrr<F>(shortcuts: &str, features: &str, f: F)
where
    F: for<'a> FnOnce(&'a WebDriver) -> BoxFuture<'a, ()>,
{
    in_browser_with(shortcuts, features, f, false, false).await;
}

async fn in_browser_with<'b, F>(
    shortcuts: &str,
    features: &str,
    f: F,
    headless: bool,
    close_browser: bool,
) where
    F: for<'a> FnOnce(&'a WebDriver) -> BoxFuture<'a, ()>,
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

    let entries = Entries::from_path(&gen_file_path(shortcuts));
    let features = GlobalFeatures::from_path(&gen_file_path(features));
    spawn(async move { server(PORT, entries, features).launch().await });

    let mut caps = DesiredCapabilities::firefox();
    if headless {
        caps.set_headless().expect("Headless conf failed");
    }

    let driver = WebDriver::new("http://localhost:4444", &caps)
        .await
        .expect("Driver build failed");

    let may_panic = AssertUnwindSafe(async { f(&driver).await });

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
