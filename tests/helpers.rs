use go::{server, Entries};
use rocket::{
    futures::{future::BoxFuture, FutureExt},
    local::blocking::Client,
    tokio::spawn,
};
use std::{collections::HashMap, panic::AssertUnwindSafe};
use thirtyfour::{prelude::WebDriverResult, DesiredCapabilities, WebDriver};

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
    F: for<'a> FnOnce(&'a WebDriver) -> BoxFuture<'a, WebDriverResult<()>>,
{
    spawn(async move { server(Entries::new(HashMap::new())).launch().await });

    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new("http://localhost:4444", &caps)
        .await
        .expect("Driver build failed");

    let may_panic = AssertUnwindSafe(async { f(&driver).await.unwrap() });

    let panics = may_panic.catch_unwind().await;

    // Always explicitly close the browser. There are no async destructors.
    // .quit() consume self
    driver.quit().await.expect("Driver quit failed");

    panics.unwrap()
}
