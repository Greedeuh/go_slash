use rocket::async_test;
use rocket::futures::FutureExt;
mod helpers;
use helpers::*;
use thirtyfour::prelude::*;

#[async_test]
async fn features_should_list_editable_features() {
    in_browser("", "[Login]", |driver: &WebDriver| {
        async {
            driver
                .get("http://localhost:8001/go/features")
                .await
                .unwrap();

            let features = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();

            assert!(!features.is_empty());

            for feature in features {
                assert_eq!(feature.text().await.unwrap(), "Login");
                let switch = feature
                    .find_element(By::Css("[role='switch']"))
                    .await
                    .unwrap();
                assert_eq!(
                    switch.get_property("checked").await.unwrap(),
                    Some("true".to_owned())
                );
                switch.click().await.unwrap();
                assert_eq!(
                    switch.get_property("checked").await.unwrap(),
                    Some("false".to_owned())
                );
            }

            driver
                .get("http://localhost:8001/go/features")
                .await
                .unwrap();

            let features = driver
                .find_elements(By::Css("[role='listitem']"))
                .await
                .unwrap();

            assert!(!features.is_empty());

            for feature in features {
                assert_eq!(feature.text().await.unwrap(), "Login");
                let switch = feature
                    .find_element(By::Css("[role='switch']"))
                    .await
                    .unwrap();
                assert_eq!(
                    switch.get_property("checked").await.unwrap(),
                    Some("false".to_owned())
                );
            }
        }
        .boxed()
    })
    .await;
}
