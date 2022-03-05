#![feature(async_closure)]
use rocket::async_test;
use rocket::futures::FutureExt;
mod helpers;
use helpers::*;
use thirtyfour::prelude::*;

#[async_test]
async fn index_should_list_shortcuts() {
    in_browser(
        "newShortcut: http://localhost:8000/newShortcut
    aShortcut: http://localhost:8000/aShortcut
    ssshortcut: http://localhost:8000/ssshortcut",
        |driver: &WebDriver| {
            async {
                let texts_sorted = vec![
                    "aShortcut http://localhost:8000/aShortcut",
                    "newShortcut http://localhost:8000/newShortcut",
                    "ssshortcut http://localhost:8000/ssshortcut",
                ];
                let href_sorted = vec![
                    "http://localhost:8000/aShortcut",
                    "http://localhost:8000/newShortcut",
                    "http://localhost:8000/ssshortcut",
                ];

                driver.get("http://localhost:8000").await.unwrap();

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
        },
    )
    .await;
}

#[async_test]
async fn index_user_as_sugestions_when_typing() {
    in_browser(
        "newShortcut: http://localhost:8000/newShortcut
    jeanLuc: http://localhost:8000/aShortcut
    tadadam: http://localhost:8000/ssshortcut",
        |driver: &WebDriver| {
            async {
                driver.get("http://localhost:8000").await.unwrap();

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
                    "tadadam http://localhost:8000/ssshortcut"
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
                    "jeanLuc http://localhost:8000/aShortcut"
                );
                assert_eq!(
                    articles[1].text().await.unwrap(),
                    "newShortcut http://localhost:8000/newShortcut"
                );
                assert_eq!(articles.len(), 2);
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn index_user_can_search() {
    in_browser(
        "newShortcut: http://localhost:8000/newShortcut
    jeanLuc: http://localhost:8000/aShortcut
    tadadam: http://localhost:8000/ssshortcut",
        |driver: &WebDriver| {
            async {
                driver.get("http://localhost:8000").await.unwrap();

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
                    "jeanLuc http://localhost:8000/aShortcut"
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
                    "jeanLuc http://localhost:8000/aShortcut"
                );
                assert!(!articles[0]
                    .class_name()
                    .await
                    .unwrap()
                    .unwrap()
                    .contains("active"));
                assert_eq!(
                    articles[1].text().await.unwrap(),
                    "newShortcut http://localhost:8000/newShortcut"
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
                    "jeanLuc http://localhost:8000/aShortcut"
                );
                assert_eq!(
                    articles[0].text().await.unwrap(),
                    "jeanLuc http://localhost:8000/aShortcut"
                );
                assert!(articles[0]
                    .class_name()
                    .await
                    .unwrap()
                    .unwrap()
                    .contains("active"));
                assert_eq!(
                    articles[1].text().await.unwrap(),
                    "newShortcut http://localhost:8000/newShortcut"
                );
                assert!(!articles[1]
                    .class_name()
                    .await
                    .unwrap()
                    .unwrap()
                    .contains("active"));

                // Tab take first
                search_bar.send_keys(Keys::Tab).await.unwrap();

                assert_eq!(
                    search_bar.get_property("value").await.unwrap(),
                    Some("jeanLuc".to_owned())
                );
            }
            .boxed()
        },
    )
    .await;
}
