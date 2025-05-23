use diesel::PgConnection;
use go_web::services::oidc::{OidcService, TokenRes};
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::http::Cookie;
use rocket::http::Status;

mod utils;
use go_web::guards::SESSION_COOKIE;
use rocket::tokio::sync::Mutex;
use thirtyfour::By;
use thirtyfour::WebDriver;
use utils::*;

mod with_google{
    use super::*;
    #[test]
    fn with_connected_user_is_not_allowed() {
        let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
        user("some_mail@mail.com", "pwd", &[], &[], &conn);
        

        let response = client
            .get("/go/login/google")
            .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[cfg(feature = "mock")]
    #[test]
    fn as_user_gen_nonce_then_redirect_to_google() {

        let mut oidc_service = OidcService::faux();


        faux::when!(oidc_service.redirect())
        .then_return(Ok(("http://auth_url".to_string(), "nonce".to_string())));


        let (client, _conn) = launch_with_sessions_and_mock("", oidc_service);
        

        let response = client.get("/go/login/google").dispatch();

        assert_eq!(response.status(), Status::PermanentRedirect);

        let mut location = response.headers().get("Location");
        let url = location.next().unwrap();
        assert_eq!(url, "http://auth_url");
        assert_eq!(location.next(), None);

        assert!(response.cookies().get(SESSION_COOKIE).is_some());
    }
}

mod after_google_redirect {
    use super::*;
    #[test]
    fn with_wrong_nonce_is_not_allowed() {
        let (client, conn) = launch_with("");
        user("some_mail@mail.com", "pwd", &[], &[], &conn);
        

        let response = client        
        .get("/go/login/redirect/google?state=4SOFn03KuR72BXkANMKnoQ&code=4%2F0AX4XfWiOsOLO15xFa1a71OykdxNOu8T-M-JaZh0dIOc3hupHDJGXUYpIx-ILk3nELtQEFw&scope=email+openid+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fuserinfo.email&authuser=1&prompt=consent").cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch(); 

        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[cfg(feature = "mock")]
    #[test]
    fn as_known_user_create_session() {

        let mut oidc_service = OidcService::faux();

        faux::when!(oidc_service.retrieve_token("code", "nonce"))
            .then_return(Ok(TokenRes {mail: "some_mail@mail.com".to_string()}));


        let (client, conn) = launch_with_sessions_and_mock("some_session_id: nonce", oidc_service);

        user("some_mail@mail.com", "pwd", &[], &[], &conn);
        

        let response = client        
        .get("/go/login/redirect/google?state=4SOFn03KuR72BXkANMKnoQ&code=code&scope=email+openid+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fuserinfo.email&authuser=1&prompt=consent").cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch(); 

        assert_eq!(response.status(), Status::PermanentRedirect);

        let mut location = response.headers().get("Location");
        let url = location.next().unwrap();
        
        assert_eq!(url, "/");
        assert!(     location.next().is_none()); 

    }

    #[cfg(feature = "mock")]
    #[test]
    fn as_unknown_user_create_user_ande_session() {
        use go_web::models::users::User;

        let mut oidc_service = OidcService::faux();
        faux::when!(oidc_service.retrieve_token("code", "nonce"))
            .then_return(Ok(TokenRes {mail: "some_mail@mail.com".to_string()}));


        let (client, conn) = launch_with_sessions_and_mock("some_session_id: nonce", oidc_service);

        

        let response = client        
        .get("/go/login/redirect/google?state=4SOFn03KuR72BXkANMKnoQ&code=code&scope=email+openid+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fuserinfo.email&authuser=1&prompt=consent").cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch(); 

        assert_eq!(response.status(), Status::PermanentRedirect);

        let mut location = response.headers().get("Location");
        let url = location.next().unwrap();
        
        assert_eq!(url, "/");
        assert!(     location.next().is_none()); 

        assert_eq!(get_user("some_mail@mail.com", &conn), Some(User {mail: "some_mail@mail.com".to_string(), capabilities: vec![]}));

    }

    #[cfg(feature = "mock")]
    #[test]
    fn as_unknown_user_with_default_capabilities_create_user_with_default_capabilities() {
        use go_web::models::users::{User, Capability};

        let mut oidc_service = OidcService::faux();
        faux::when!(oidc_service.retrieve_token("code", "nonce"))
            .then_return(Ok(TokenRes {mail: "some_mail@mail.com".to_string()}));


        let (client, conn) = launch_with_sessions_and_mock("some_session_id: nonce", oidc_service);

        default_capabilities(&[ Capability::TeamsWriteWithValidation],&conn);
        

        let response = client        
        .get("/go/login/redirect/google?state=4SOFn03KuR72BXkANMKnoQ&code=code&scope=email+openid+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fuserinfo.email&authuser=1&prompt=consent").cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch(); 

        assert_eq!(response.status(), Status::PermanentRedirect);

        let mut location = response.headers().get("Location");
        let url = location.next().unwrap();
        
        assert_eq!(url, "/");
        assert!(     location.next().is_none()); 

        assert_eq!(get_user("some_mail@mail.com", &conn), Some(User {mail: "some_mail@mail.com".to_string(), capabilities: vec![ Capability::TeamsWriteWithValidation]}));

    }
}

#[async_test]
async fn login_page_has_oauth_links() {
    in_browser(
        "",
        |driver: &WebDriver, _con: Mutex<PgConnection>, port: u16| {
            async move {
               

                driver
                    .get(format!("http://host.docker.internal:{}/go/login?from=allo", port))
                    .await?;
                

                assert_eq!(
                    driver
                        .find_element(By::Css("a[aria-label='Login with google']"))
                        .await?
                        .text()
                        .await?,
                    "Login with google"
                );

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}
