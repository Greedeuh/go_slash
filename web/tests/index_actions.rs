use diesel::PgConnection;
use go_web::guards::SESSION_COOKIE;
use go_web::models::features::{Features, LoginFeature};
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
use serde_json::json;
use thirtyfour::components::select::SelectElement;
mod utils;
use thirtyfour::prelude::*;
use utils::*;

#[async_test]
async fn index_user_can_delete_shortcuts() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
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
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
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
        |driver: &WebDriver, _con: Mutex<PgConnection>, port: u16| {
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
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
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
