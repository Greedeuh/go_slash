use go::{server, Entries};
use rocket::{
    futures::{future::BoxFuture, FutureExt},
    local::blocking::Client,
    tokio::spawn,
};
use std::{
    collections::HashMap,
    panic::{resume_unwind, AssertUnwindSafe},
};
use thirtyfour::{DesiredCapabilities, WebDriver};

pub fn launch_empty() -> Client {
    Client::tracked(server(Entries::new(HashMap::new()))).expect("valid rocket instance")
}

#[allow(dead_code)]
pub fn launch_with(shortcuts: &str) -> Client {
    let shortcuts = shortcuts
        .lines()
        .map(|line| {
            let line = line.replace(' ', "");
            let (key, value) = line
                .split_once(':')
                .expect("launch_with shortcuts failed parsing");
            (key.to_owned(), value.to_owned())
        })
        .collect();

    Client::tracked(server(Entries::new(shortcuts))).expect("valid rocket instance")
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
pub async fn in_browser<F>(f: F)
where
    F: for<'a> FnOnce(&'a WebDriver) -> BoxFuture<'a, ()>,
{
    in_browser_with(f, true).await;
}

#[allow(dead_code)]
/// Same but launch browser
pub async fn in_browserr<F>(f: F)
where
    F: for<'a> FnOnce(&'a WebDriver) -> BoxFuture<'a, ()>,
{
    in_browser_with(f, false).await;
}

async fn in_browser_with<F>(f: F, headless: bool)
where
    F: for<'a> FnOnce(&'a WebDriver) -> BoxFuture<'a, ()>,
{
    spawn(async move { server(Entries::new(HashMap::new())).launch().await });

    let mut caps = DesiredCapabilities::firefox();
    if headless {
        caps.set_headless().expect("Headless conf failed");
    }

    let driver = WebDriver::new("http://localhost:4444", &caps)
        .await
        .expect("Driver build failed");

    let may_panic = AssertUnwindSafe(async { f(&driver).await });

    let maybe_panicked = may_panic.catch_unwind().await;

    // Always explicitly close the browser. There are no async destructors.
    if headless {
        driver.quit().await.expect("Driver quit failed");
    }

    if let Err(panic) = maybe_panicked {
        resume_unwind(panic)
    }
}
