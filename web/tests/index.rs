use diesel::SqliteConnection;
use go_web::guards::SESSION_COOKIE;
use go_web::models::features::{Features, LoginFeature};
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
use serde_json::json;
use std::thread;
use std::time::Duration;
use thirtyfour::components::select::SelectElement;
mod utils;
use thirtyfour::prelude::*;
use utils::*;

#[async_test]
async fn index_should_list_shortcuts() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                team("team1", "Team 1", false, true, &con);
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/newShortcut", port),
                    "",
                    &con,
                );
                shortcut(
                    "aShortcut",
                    &format!("http://localhost:{}/aShortcut", port),
                    "team1",
                    &con,
                );
                shortcut(
                    "ssshortcut",
                    &format!("http://localhost:{}/ssshortcut", port),
                    "",
                    &con,
                );

                let texts_sorted = vec![
                    format!("aShortcut http://localhost:{}/aShortcut team1", port),
                    format!("newShortcut http://localhost:{}/newShortcut", port),
                    format!("ssshortcut http://localhost:{}/ssshortcut", port),
                ];
                let href_sorted = vec![
                    format!("http://localhost:{}/aShortcut", port),
                    format!("http://localhost:{}/newShortcut", port),
                    format!("http://localhost:{}/ssshortcut", port),
                ];

                driver.get(format!("http://localhost:{}", port)).await?;

                let login_link = driver.find_element(By::Css("a.nav-link")).await?;
                assert_eq!(
                    login_link.get_attribute("href").await?,
                    Some("/go/login".to_owned())
                );
                assert_eq!(login_link.text().await?, "Login");

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;

                for i in 0..texts_sorted.len() {
                    assert_eq!(articles[i].text().await?, texts_sorted[i]);
                    assert_eq!(
                        articles[i].get_attribute("href").await?,
                        Some(href_sorted[i].to_owned())
                    );
                }
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn index_user_as_sugestions_when_typing() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                team("team1", "Team 1", false, true, &con);
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/newShortcut", port),
                    "",
                    &con,
                );
                shortcut(
                    "jeanLuc",
                    &format!("http://localhost:{}/aShortcut", port),
                    "slug1",
                    &con,
                );
                shortcut(
                    "tadadam",
                    &format!("http://localhost:{}/ssshortcut", port),
                    "",
                    &con,
                );

                driver.get(format!("http://localhost:{}", port)).await?;

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                // initial state
                assert_eq!(3, articles.len());

                let search_bar = driver.find_element(By::Css("input[type='search']")).await?;
                search_bar.send_keys("t").await?;

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;

                // type in t should suggest tadadam first
                assert_eq!(
                    articles[0].text().await?,
                    format!("tadadam http://localhost:{}/ssshortcut", port)
                );
                assert_eq!(articles.len(), 3);

                search_bar.send_keys("uc").await?;

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;

                // type in tuc should suggest jeanLuc and newShortcut but not tadam
                assert_eq!(
                    articles[0].text().await?,
                    format!("jeanLuc http://localhost:{}/aShortcut slug1", port)
                );
                assert_eq!(
                    articles[1].text().await?,
                    format!("newShortcut http://localhost:{}/newShortcut", port)
                );
                assert_eq!(articles.len(), 2);
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn index_user_can_search() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/newShortcut", port),
                    "",
                    &con,
                );
                shortcut(
                    "jeanLuc",
                    &format!("http://localhost:{}/aShortcut1", port),
                    "",
                    &con,
                );
                shortcut(
                    "tadadam",
                    &format!("http://localhost:{}/ssshortcut", port),
                    "",
                    &con,
                );

                driver.get(format!("http://localhost:{}", port)).await?;

                let search_bar = driver.find_element(By::Css("input[type='search']")).await?;
                search_bar.send_keys(Keys::Down).await?;

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;

                // down arrow select first
                assert_eq!(
                    articles[0].text().await?,
                    format!("jeanLuc http://localhost:{}/aShortcut1", port)
                );
                assert!(articles[0].class_name().await?.unwrap().contains("active"));

                search_bar.send_keys(Keys::Down).await?;

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;

                // down arrow again select snd & unselect first
                assert_eq!(
                    articles[0].text().await?,
                    format!("jeanLuc http://localhost:{}/aShortcut1", port)
                );
                assert!(!articles[0].class_name().await?.unwrap().contains("active"));
                assert_eq!(
                    articles[1].text().await?,
                    format!("newShortcut http://localhost:{}/newShortcut", port)
                );
                assert!(articles[1].class_name().await?.unwrap().contains("active"));

                search_bar.send_keys(Keys::Up).await?;

                // up arrow select first & unselect first
                assert_eq!(
                    articles[0].text().await?,
                    format!("jeanLuc http://localhost:{}/aShortcut1", port)
                );
                assert!(articles[0].class_name().await?.unwrap().contains("active"));
                assert_eq!(
                    articles[1].text().await?,
                    format!("newShortcut http://localhost:{}/newShortcut", port)
                );
                assert!(!articles[1].class_name().await?.unwrap().contains("active"));

                search_bar.send_keys(Keys::Tab).await?;

                // Tab take first
                assert_eq!(
                    search_bar.get_property("value").await?,
                    Some("jeanLuc".to_owned())
                );

                search_bar.send_keys(Keys::Enter).await?;
                sleep();

                // Enter launch search
                assert_eq!(
                    driver.current_url().await?,
                    format!("http://localhost:{}/aShortcut1", port)
                );

                driver.get(format!("http://localhost:{}", port)).await?;
                // arow down then enter go to the first line shortcut

                let search_bar = driver.find_element(By::Css("input[type='search']")).await?;
                search_bar.send_keys(Keys::Down).await?;
                search_bar.send_keys(Keys::Enter).await?;

                sleep();

                assert_eq!(
                    driver.current_url().await?,
                    format!("http://localhost:{}/aShortcut1", port)
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn index_user_can_delete_shortcuts() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/newShortcut", port),
                    "",
                    &con,
                );

                driver.get(format!("http://localhost:{}", port)).await?;

                let administer_btn = driver.find_element(By::Id("btn-administer")).await?;
                assert_eq!(
                    administer_btn.class_name().await?,
                    Some("btn-light btn".to_owned())
                );
                administer_btn.click().await?;

                let delete_btn = driver.find_element(By::Id("btn-delete")).await?;

                // Escape should quit the admin mode
                administer_btn.send_keys(Keys::Escape).await?;
                assert!(!delete_btn.is_present().await?);

                administer_btn.click().await?;
                let delete_btn = driver.find_element(By::Id("btn-delete")).await?;
                delete_btn.click().await?;

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(articles.len(), 0);

                let search_bar = driver.find_element(By::Css("input[type='search']")).await?;
                search_bar.send_keys("newShortcut").await?;
                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(articles.len(), 0);

                driver.get(format!("http://localhost:{}", port)).await?;
                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(articles.len(), 0);
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn index_user_can_delete_shortcuts_with_team() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                team("team1", "Team 1", false, true, &con);
                shortcut(
                    "jeanLuc",
                    &format!("http://localhost:{}/aShortcut1", port),
                    "team1",
                    &con,
                );
                user(
                    "some_mail@mail.com",
                    "pwd",
                    true,
                    &[("team1", true, 0)],
                    &con,
                );
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            ..Default::default()
                        },
                        teams: true,
                    },
                    &con,
                );
                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                    .await?;
                driver.get(format!("http://localhost:{}", port)).await?;

                let administer_btn = driver.find_element(By::Id("btn-administer")).await?;
                assert_eq!(
                    administer_btn.class_name().await?,
                    Some("btn-light btn".to_owned())
                );

                administer_btn.click().await?;
                driver
                    .find_element(By::Id("btn-delete"))
                    .await?
                    .click()
                    .await?;

                sleep();

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(articles.len(), 0);

                driver.get(format!("http://localhost:{}", port)).await?;
                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(articles.len(), 0);
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn index_user_can_add_shortcuts() {
    in_browser(
        "",
        |driver: &WebDriver, _con: Mutex<SqliteConnection>, port: u16| {
            async move {
                driver.get(format!("http://localhost:{}", port)).await?;

                let administer_btn = driver.find_element(By::Id("btn-administer")).await?;
                assert_eq!(
                    administer_btn.class_name().await?,
                    Some("btn-light btn".to_owned())
                );
                administer_btn.click().await?;

                driver
                    .find_element(By::Css("[name='shortcut']"))
                    .await?
                    .send_keys("jeanLuc")
                    .await?;
                driver
                    .find_element(By::Css("[name='url']"))
                    .await?
                    .send_keys(format!("http://localhost:{}/aShortcut", port))
                    .await?;

                // no team feature
                assert!(driver.find_element(By::Css("[name='team']")).await.is_err());

                driver
                    .find_element(By::Id("btn-add"))
                    .await?
                    .click()
                    .await?;

                let article = driver.find_element(By::Css("[role='listitem']")).await?;
                assert_eq!(
                    article.text().await?,
                    format!("jeanLuc http://localhost:{}/aShortcut NEW", port)
                );

                assert_eq!(
                    article.get_property("href").await?,
                    Some(format!("http://localhost:{}/jeanLuc?no_redirect", port))
                );

                assert_eq!(
                    driver
                        .find_element(By::Css("[name='shortcut']"))
                        .await?
                        .get_property("value")
                        .await?,
                    Some("".to_owned())
                );
                assert_eq!(
                    driver
                        .find_element(By::Css("[name='url']"))
                        .await?
                        .get_property("value")
                        .await?,
                    Some("".to_owned())
                );

                administer_btn.click().await?;
                assert_eq!(
                    article.get_property("href").await?,
                    Some(format!("http://localhost:{}/aShortcut", port))
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn index_user_can_add_shortcuts_for_team() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                team("slug1", "team1", false, false, &con);
                team("slug2", "team1", false, false, &con);
                user(
                    "some_mail@mail.com",
                    "pwd",
                    false,
                    &[("slug1", true, 0), ("slug2", true, 0)],
                    &con,
                );
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            ..Default::default()
                        },
                        teams: true,
                    },
                    &con,
                );

                let options_expected = vec!["slug1", "slug2"];

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, json!("some_session_id")))
                    .await?;

                driver.get(format!("http://localhost:{}", port)).await?;

                let administer_btn = driver.find_element(By::Id("btn-administer")).await?;
                assert_eq!(
                    administer_btn.class_name().await?,
                    Some("btn-light btn".to_owned())
                );
                administer_btn.click().await?;

                driver
                    .find_element(By::Css("[name='shortcut']"))
                    .await?
                    .send_keys("jeanLuc")
                    .await?;
                driver
                    .find_element(By::Css("[name='url']"))
                    .await?
                    .send_keys(format!("http://localhost:{}/aShortcut", port))
                    .await?;

                // no team feature
                let team = driver.find_element(By::Css("[name='team']")).await?;
                let team = SelectElement::new(&team).await?;

                let options = team.options().await?;
                for i in 0..options_expected.len() {
                    assert_eq!(
                        options[i].get_attribute("value").await?,
                        Some(options_expected[i].to_string())
                    );
                    assert_eq!(options[i].text().await?, options_expected[i]);
                }

                team.select_by_exact_text("slug1").await?;

                driver
                    .find_element(By::Id("btn-add"))
                    .await?
                    .click()
                    .await?;

                sleep();

                let article = driver.find_element(By::Css("[role='listitem']")).await?;
                assert_eq!(
                    article.text().await?,
                    format!("jeanLuc http://localhost:{}/aShortcut slug1NEW", port)
                );

                assert_eq!(
                    article.get_property("href").await?,
                    Some(format!("http://localhost:{}/jeanLuc?no_redirect", port))
                );

                assert_eq!(
                    driver
                        .find_element(By::Css("[name='shortcut']"))
                        .await?
                        .get_property("value")
                        .await?,
                    Some("".to_owned())
                );

                assert_eq!(
                    driver
                        .find_element(By::Css("[name='team']"))
                        .await?
                        .get_property("value")
                        .await?,
                    Some("slug1".to_owned())
                );

                administer_btn.click().await?;
                assert_eq!(
                    article.get_property("href").await?,
                    Some(format!("http://localhost:{}/aShortcut", port))
                );

                driver.get(format!("http://localhost:{}", port)).await?;
                let article = driver.find_element(By::Css("[role='listitem']")).await?;
                assert_eq!(
                    article.text().await?,
                    format!("jeanLuc http://localhost:{}/aShortcut slug1", port)
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn shortcut_no_redirect_return_search_filled_and_edit_form() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                shortcut(
                    "newShortcut",
                    &format!("http://localhost:{}/looped", port),
                    "",
                    &con,
                );
                shortcut(
                    "newShortcut2",
                    &format!("http://localhost:{}/claude", port),
                    "",
                    &con,
                );

                // create shortcut
                driver
                    .get(format!(
                        "http://localhost:{}/newShortcut?no_redirect=true",
                        port
                    ))
                    .await?;

                let login_link = driver.find_element(By::Css("a.nav-link")).await?;
                assert_eq!(
                    login_link.get_attribute("href").await?,
                    Some("/go/login".to_owned())
                );
                assert_eq!(login_link.text().await?, "Login");

                let search_bar = driver.find_element(By::Css("input[type='search']")).await?;
                assert_eq!(
                    search_bar.get_property("value").await?,
                    Some("newShortcut".to_owned())
                );

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("newShortcut http://localhost:{}/looped", port)
                );
                assert_eq!(articles.len(), 2);

                let shortcut_edit = driver
                    .find_element(By::Css("input[name='shortcut']"))
                    .await?;
                assert_eq!(
                    shortcut_edit.get_property("value").await?,
                    Some("newShortcut".to_owned())
                );
                assert_eq!(
                    shortcut_edit.get_property("disabled").await?,
                    Some("true".to_owned())
                );

                driver
                    .find_element(By::Css("input[name='url']"))
                    .await?
                    .send_keys("2")
                    .await?;
                driver
                    .find_element(By::Id("btn-add"))
                    .await?
                    .click()
                    .await?;

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("newShortcut http://localhost:{}/looped2 NEW", port)
                );
                assert_eq!(articles.len(), 2);
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn undefined_shortcut_return_search_filled_and_edit_form() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let con = con.lock().await;
                shortcut(
                    "newShortcut1",
                    &format!("http://localhost:{}/looped", port),
                    "",
                    &con,
                );
                shortcut(
                    "newShortcut2",
                    &format!("http://localhost:{}/claude", port),
                    "",
                    &con,
                );

                // create shortcut
                driver
                    .get(format!("http://localhost:{}/newShortcut", port))
                    .await?;

                let search_bar = driver.find_element(By::Css("input[type='search']")).await?;
                assert_eq!(
                    search_bar.get_property("value").await?,
                    Some("newShortcut".to_owned())
                );

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("newShortcut1 http://localhost:{}/looped", port)
                );
                assert_eq!(articles.len(), 2);

                let shortcut_edit = driver
                    .find_element(By::Css("input[name='shortcut']"))
                    .await?;
                assert_eq!(
                    shortcut_edit.get_property("value").await?,
                    Some("newShortcut".to_owned())
                );
                assert_eq!(
                    shortcut_edit.get_property("disabled").await?,
                    Some("true".to_owned())
                );

                driver
                    .find_element(By::Css("input[name='url']"))
                    .await?
                    .send_keys(format!("http://localhost:{}/ring", port))
                    .await?;
                driver
                    .find_element(By::Id("btn-add"))
                    .await?
                    .click()
                    .await?;

                let articles = driver.find_elements(By::Css("[role='listitem']")).await?;
                assert_eq!(
                    articles[0].text().await?,
                    format!("newShortcut http://localhost:{}/ring NEW", port)
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn not_logged_in_should_redirect_to_login() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let conn = con.lock().await;
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            read_private: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    &conn,
                );

                driver.get(format!("http://localhost:{}", port)).await?;
                thread::sleep(Duration::from_secs_f32(0.6));
                assert_eq!(
                    driver.current_url().await?,
                    format!("http://localhost:{}/go/login", port)
                );

                driver
                    .get(format!("http://localhost:{}/shortcut", port))
                    .await?;
                thread::sleep(Duration::from_secs_f32(0.6));
                assert_eq!(
                    driver.current_url().await?,
                    format!("http://localhost:{}/go/login?from=/shortcut", port)
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn logged_in_without_write() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<SqliteConnection>, port: u16| {
            async move {
                let conn = con.lock().await;
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            write_private: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    &conn,
                );

                driver.get(format!("http://localhost:{}", port)).await?;
                assert_eq!(
                    0,
                    driver.find_elements(By::Id("btn-administer")).await?.len()
                );

                driver
                    .get(format!("http://localhost:{}/shortcut", port))
                    .await?;
                assert_eq!(
                    0,
                    driver.find_elements(By::Id("btn-administer")).await?.len()
                );
                assert_eq!(
                    0,
                    driver.find_elements(By::Id("btn-administer")).await?.len()
                );
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}
