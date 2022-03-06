use go::{server, Entries};
use rocket::{
    futures::{future::BoxFuture, FutureExt},
    local::blocking::Client,
    tokio::spawn,
};
use std::{
    collections::HashMap,
    env,
    panic::{resume_unwind, AssertUnwindSafe},
    time::Duration,
};
use thirtyfour::{DesiredCapabilities, WebDriver};

#[allow(dead_code)]
pub fn launch_empty() -> Client {
    Client::tracked(server(Entries::new(HashMap::new()))).expect("valid rocket instance")
}

#[allow(dead_code)]
pub fn launch_with(shortcuts: &str) -> Client {
    Client::tracked(server(Entries::from(shortcuts))).expect("valid rocket instance")
}

#[allow(dead_code)]
pub fn entries(shortcuts: &str) -> Entries {
    Entries::new(
        shortcuts
            .lines()
            .map(|line| {
                let line = line.replace(' ', "");
                let (key, value) = line
                    .split_once(':')
                    .expect("launch_with shortcuts failed parsing");
                (key.to_owned(), value.to_owned())
            })
            .collect(),
    )
}

#[allow(dead_code)]
pub async fn in_browser<F>(shortcuts: &str, f: F)
where
    F: for<'a> FnOnce(&'a WebDriver) -> BoxFuture<'a, ()>,
{
    in_browser_with(shortcuts, f, true, true).await;
}

#[allow(dead_code)]
/// Same but launch browser
#[deprecated(note = "Should only be used in local")]
pub async fn in_browserr<F>(shortcuts: &str, f: F)
where
    F: for<'a> FnOnce(&'a WebDriver) -> BoxFuture<'a, ()>,
{
    in_browser_with(shortcuts, f, false, true).await;
}

#[allow(dead_code)]
/// Same but launch browser and do not kill it
#[deprecated(note = "Should only be used in local")]
pub async fn in_browserrr<F>(shortcuts: &str, f: F)
where
    F: for<'a> FnOnce(&'a WebDriver) -> BoxFuture<'a, ()>,
{
    in_browser_with(shortcuts, f, false, false).await;
}

async fn in_browser_with<'b, F>(shortcuts: &str, f: F, headless: bool, close_browser: bool)
where
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

    let entries = Entries::from(shortcuts);
    spawn(async move { server(entries).launch().await });

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
