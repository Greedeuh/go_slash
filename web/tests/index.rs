use diesel::SqliteConnection;
use go_web::models::features::{Features, LoginFeature};
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
use std::thread;
use std::time::Duration;
mod utils;
use thirtyfour::prelude::*;
use utils::*;

#[async_test]
async fn index_should_list_shortcuts() {
    in_browser("", |driver: &WebDriver, con: Mutex<SqliteConnection>| {
        async move {
            let con = con.lock().await;
            shortcut("newShortcut", "http://localhost:8001/newShortcut", "", &con);
            shortcut("aShortcut", "http://localhost:8001/aShortcut", "", &con);
            shortcut("ssshortcut", "http://localhost:8001/ssshortcut", "", &con);

            let texts_sorted = vec![
                "aShortcut http://localhost:8001/aShortcut",
                "newShortcut http://localhost:8001/newShortcut",
                "ssshortcut http://localhost:8001/ssshortcut",
            ];
            let href_sorted = vec![
                "http://localhost:8001/aShortcut",
                "http://localhost:8001/newShortcut",
                "http://localhost:8001/ssshortcut",
            ];

            driver.get("http://localhost:8001").await.unwrap();

            let login_link = driver.find_element(By::Css("a.nav-link")).await.unwrap();
            assert_eq!(
                login_link.get_attribute("href").await.unwrap(),
                Some("/go/login".to_owned())
            );
            assert_eq!(login_link.text().await.unwrap(), "Login");

            let articles = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();

            for i in 0..texts_sorted.len() {
                assert_eq!(&articles[i].text().await.unwrap(), texts_sorted[i]);
                assert_eq!(
                    articles[i].get_attribute("href").await.unwrap(),
                    Some(href_sorted[i].to_owned())
                );
            }
        }
        .boxed()
    })
    .await;
}

#[async_test]
async fn index_user_as_sugestions_when_typing() {
    in_browser("", |driver: &WebDriver, con: Mutex<SqliteConnection>| {
        async move {
            let con = con.lock().await;
            shortcut("newShortcut", "http://localhost:8001/newShortcut", "", &con);
            shortcut("jeanLuc", "http://localhost:8001/aShortcut", "", &con);
            shortcut("tadadam", "http://localhost:8001/ssshortcut", "", &con);

            driver.get("http://localhost:8001").await.unwrap();

            let articles = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();
            // initial state
            assert_eq!(3, articles.len());

            let search_bar = driver
                .find_element(By::Css("input[type='search']"))
                .await
                .unwrap();
            search_bar.send_keys("t").await.unwrap();

            let articles = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();

            // type in t should suggest tadadam first
            assert_eq!(
                articles[0].text().await.unwrap(),
                "tadadam http://localhost:8001/ssshortcut"
            );
            assert_eq!(articles.len(), 3);

            search_bar.send_keys("uc").await.unwrap();

            let articles = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();

            // type in tuc should suggest jeanLuc and newShortcut but not tadam
            assert_eq!(
                articles[0].text().await.unwrap(),
                "jeanLuc http://localhost:8001/aShortcut"
            );
            assert_eq!(
                articles[1].text().await.unwrap(),
                "newShortcut http://localhost:8001/newShortcut"
            );
            assert_eq!(articles.len(), 2);
        }
        .boxed()
    })
    .await;
}

#[async_test]
async fn index_user_can_search() {
    in_browser("", |driver: &WebDriver, con: Mutex<SqliteConnection>| {
        async move {
            let con = con.lock().await;
            shortcut("newShortcut", "http://localhost:8001/newShortcut", "", &con);
            shortcut("jeanLuc", "http://localhost:8001/aShortcut1", "", &con);
            shortcut("tadadam", "http://localhost:8001/ssshortcut", "", &con);

            driver.get("http://localhost:8001").await.unwrap();

            let search_bar = driver
                .find_element(By::Css("input[type='search']"))
                .await
                .unwrap();
            search_bar.send_keys(Keys::Down).await.unwrap();

            let articles = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();

            // down arrow select first
            assert_eq!(
                articles[0].text().await.unwrap(),
                "jeanLuc http://localhost:8001/aShortcut1"
            );
            assert!(articles[0]
                .class_name()
                .await
                .unwrap()
                .unwrap()
                .contains("active"));

            search_bar.send_keys(Keys::Down).await.unwrap();

            let articles = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();

            // down arrow again select snd & unselect first
            assert_eq!(
                articles[0].text().await.unwrap(),
                "jeanLuc http://localhost:8001/aShortcut1"
            );
            assert!(!articles[0]
                .class_name()
                .await
                .unwrap()
                .unwrap()
                .contains("active"));
            assert_eq!(
                articles[1].text().await.unwrap(),
                "newShortcut http://localhost:8001/newShortcut"
            );
            assert!(articles[1]
                .class_name()
                .await
                .unwrap()
                .unwrap()
                .contains("active"));

            search_bar.send_keys(Keys::Up).await.unwrap();

            // up arrow select first & unselect first
            assert_eq!(
                articles[0].text().await.unwrap(),
                "jeanLuc http://localhost:8001/aShortcut1"
            );
            assert!(articles[0]
                .class_name()
                .await
                .unwrap()
                .unwrap()
                .contains("active"));
            assert_eq!(
                articles[1].text().await.unwrap(),
                "newShortcut http://localhost:8001/newShortcut"
            );
            assert!(!articles[1]
                .class_name()
                .await
                .unwrap()
                .unwrap()
                .contains("active"));

            search_bar.send_keys(Keys::Tab).await.unwrap();

            // Tab take first
            assert_eq!(
                search_bar.get_property("value").await.unwrap(),
                Some("jeanLuc".to_owned())
            );

            search_bar.send_keys(Keys::Enter).await.unwrap();
            sleep();

            // Enter launch search
            assert_eq!(
                driver.current_url().await.unwrap(),
                "http://localhost:8001/aShortcut1"
            );

            driver.get("http://localhost:8001").await.unwrap();
            // arow down then enter go to the first line shortcut

            let search_bar = driver
                .find_element(By::Css("input[type='search']"))
                .await
                .unwrap();
            search_bar.send_keys(Keys::Down).await.unwrap();
            search_bar.send_keys(Keys::Enter).await.unwrap();

            sleep();

            assert_eq!(
                driver.current_url().await.unwrap(),
                "http://localhost:8001/aShortcut1"
            );
        }
        .boxed()
    })
    .await;
}

#[async_test]
async fn index_user_can_delete_shortcuts() {
    in_browser("", |driver: &WebDriver, con: Mutex<SqliteConnection>| {
        async move {
            let con = con.lock().await;
            shortcut("newShortcut", "http://localhost:8001/newShortcut", "", &con);

            driver.get("http://localhost:8001").await.unwrap();

            let administer_btn = driver.find_element(By::Id("btn-administer")).await.unwrap();
            assert_eq!(
                administer_btn.class_name().await.unwrap(),
                Some("btn-light btn".to_owned())
            );
            administer_btn.click().await.unwrap();

            let delete_btn = driver.find_element(By::Id("btn-delete")).await.unwrap();

            // Escape should quit the admin mode
            administer_btn.send_keys(Keys::Escape).await.unwrap();
            assert!(!delete_btn.is_present().await.unwrap());

            administer_btn.click().await.unwrap();
            let delete_btn = driver.find_element(By::Id("btn-delete")).await.unwrap();
            delete_btn.click().await.unwrap();

            sleep();

            let articles = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();
            assert_eq!(articles.len(), 0);

            let search_bar = driver
                .find_element(By::Css("input[type='search']"))
                .await
                .unwrap();
            search_bar.send_keys("newShortcut").await.unwrap();
            let articles = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();
            assert_eq!(articles.len(), 0);

            driver.get("http://localhost:8001").await.unwrap();
            let articles = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();
            assert_eq!(articles.len(), 0);
        }
        .boxed()
    })
    .await;
}

#[async_test]
async fn index_user_can_add_shortcuts() {
    in_browser("", |driver: &WebDriver, _con: Mutex<SqliteConnection>| {
        async {
            driver.get("http://localhost:8001").await.unwrap();

            let administer_btn = driver.find_element(By::Id("btn-administer")).await.unwrap();
            assert_eq!(
                administer_btn.class_name().await.unwrap(),
                Some("btn-light btn".to_owned())
            );
            administer_btn.click().await.unwrap();

            driver
                .find_element(By::Css("[name='shortcut']"))
                .await
                .unwrap()
                .send_keys("jeanLuc")
                .await
                .unwrap();
            driver
                .find_element(By::Css("[name='url']"))
                .await
                .unwrap()
                .send_keys("http://localhost:8001/aShortcut")
                .await
                .unwrap();
            driver
                .find_element(By::Id("btn-add"))
                .await
                .unwrap()
                .click()
                .await
                .unwrap();

            sleep();

            let article = driver
                .find_element(By::Css("[role='listitem']"))
                .await
                .unwrap();
            assert_eq!(
                article.text().await.unwrap(),
                "jeanLuc http://localhost:8001/aShortcut NEW"
            );

            assert_eq!(
                article.get_property("href").await.unwrap(),
                Some("http://localhost:8001/jeanLuc?no_redirect".to_owned())
            );

            assert_eq!(
                driver
                    .find_element(By::Css("[name='shortcut']"))
                    .await
                    .unwrap()
                    .get_property("value")
                    .await
                    .unwrap(),
                Some("".to_owned())
            );
            assert_eq!(
                driver
                    .find_element(By::Css("[name='url']"))
                    .await
                    .unwrap()
                    .get_property("value")
                    .await
                    .unwrap(),
                Some("".to_owned())
            );

            administer_btn.click().await.unwrap();
            assert_eq!(
                article.get_property("href").await.unwrap(),
                Some("http://localhost:8001/aShortcut".to_owned())
            );
        }
        .boxed()
    })
    .await;
}

#[async_test]
async fn shortcut_no_redirect_return_search_filled_and_edit_form() {
    in_browser("", |driver: &WebDriver, con: Mutex<SqliteConnection>| {
        async move {
            let con = con.lock().await;
            shortcut("newShortcut", "http://localhost:8001/looped", "", &con);
            shortcut("newShortcut2", "http://localhost:8001/claude", "", &con);

            // create shortcut
            driver
                .get("http://localhost:8001/newShortcut?no_redirect=true")
                .await
                .unwrap();

            let login_link = driver.find_element(By::Css("a.nav-link")).await.unwrap();
            assert_eq!(
                login_link.get_attribute("href").await.unwrap(),
                Some("/go/login".to_owned())
            );
            assert_eq!(login_link.text().await.unwrap(), "Login");

            let search_bar = driver
                .find_element(By::Css("input[type='search']"))
                .await
                .unwrap();
            assert_eq!(
                search_bar.get_property("value").await.unwrap(),
                Some("newShortcut".to_owned())
            );

            let articles = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();
            assert_eq!(
                articles[0].text().await.unwrap(),
                "newShortcut http://localhost:8001/looped"
            );
            assert_eq!(articles.len(), 2);

            let shortcut_edit = driver
                .find_element(By::Css("input[name='shortcut']"))
                .await
                .unwrap();
            assert_eq!(
                shortcut_edit.get_property("value").await.unwrap(),
                Some("newShortcut".to_owned())
            );
            assert_eq!(
                shortcut_edit.get_property("disabled").await.unwrap(),
                Some("true".to_owned())
            );

            driver
                .find_element(By::Css("input[name='url']"))
                .await
                .unwrap()
                .send_keys("2")
                .await
                .unwrap();
            driver
                .find_element(By::Id("btn-add"))
                .await
                .unwrap()
                .click()
                .await
                .unwrap();

            let articles = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();
            assert_eq!(
                articles[0].text().await.unwrap(),
                "newShortcut http://localhost:8001/looped2 NEW"
            );
            assert_eq!(articles.len(), 2);
        }
        .boxed()
    })
    .await;
}

#[async_test]
async fn undefined_shortcut_return_search_filled_and_edit_form() {
    in_browser("", |driver: &WebDriver, con: Mutex<SqliteConnection>| {
        async move {
            let con = con.lock().await;
            shortcut("newShortcut1", "http://localhost:8001/looped", "", &con);
            shortcut("newShortcut2", "http://localhost:8001/claude", "", &con);

            // create shortcut
            driver
                .get("http://localhost:8001/newShortcut")
                .await
                .unwrap();

            let search_bar = driver
                .find_element(By::Css("input[type='search']"))
                .await
                .unwrap();
            assert_eq!(
                search_bar.get_property("value").await.unwrap(),
                Some("newShortcut".to_owned())
            );

            let articles = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();
            assert_eq!(
                articles[0].text().await.unwrap(),
                "newShortcut1 http://localhost:8001/looped"
            );
            assert_eq!(articles.len(), 2);

            let shortcut_edit = driver
                .find_element(By::Css("input[name='shortcut']"))
                .await
                .unwrap();
            assert_eq!(
                shortcut_edit.get_property("value").await.unwrap(),
                Some("newShortcut".to_owned())
            );
            assert_eq!(
                shortcut_edit.get_property("disabled").await.unwrap(),
                Some("true".to_owned())
            );

            driver
                .find_element(By::Css("input[name='url']"))
                .await
                .unwrap()
                .send_keys("http://localhost:8001/ring")
                .await
                .unwrap();
            driver
                .find_element(By::Id("btn-add"))
                .await
                .unwrap()
                .click()
                .await
                .unwrap();

            let articles = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();
            assert_eq!(
                articles[0].text().await.unwrap(),
                "newShortcut http://localhost:8001/ring NEW"
            );
        }
        .boxed()
    })
    .await;
}

#[async_test]
async fn not_logged_in_should_redirect_to_login() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<SqliteConnection>| {
            async move {
                let conn = con.lock().await;
                global_features(
                    &Features {
                        login: LoginFeature {
                            simple: true,
                            read_private: true,
                            ..Default::default()
                        },
                    },
                    &conn,
                );

                driver.get("http://localhost:8001").await.unwrap();
                thread::sleep(Duration::from_secs_f32(0.6));
                assert_eq!(
                    driver.current_url().await.unwrap(),
                    "http://localhost:8001/go/login"
                );

                driver.get("http://localhost:8001/shortcut").await.unwrap();
                thread::sleep(Duration::from_secs_f32(0.6));
                assert_eq!(
                    driver.current_url().await.unwrap(),
                    "http://localhost:8001/go/login?from=/shortcut"
                );
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn logged_in_without_write() {
    in_browser("", |driver: &WebDriver, con: Mutex<SqliteConnection>| {
        async move {
            let conn = con.lock().await;
            global_features(
                &Features {
                    login: LoginFeature {
                        simple: true,
                        write_private: true,
                        ..Default::default()
                    },
                },
                &conn,
            );

            driver.get("http://localhost:8001").await.unwrap();
            assert_eq!(
                0,
                driver
                    .find_elements(By::Id("btn-administer"))
                    .await
                    .unwrap()
                    .len()
            );

            driver.get("http://localhost:8001/shortcut").await.unwrap();
            assert_eq!(
                0,
                driver
                    .find_elements(By::Id("btn-administer"))
                    .await
                    .unwrap()
                    .len()
            );
            assert_eq!(
                0,
                driver
                    .find_elements(By::Id("btn-administer"))
                    .await
                    .unwrap()
                    .len()
            );
        }
        .boxed()
    })
    .await;
}
