use diesel::SqliteConnection;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
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