use diesel::PgConnection;
use go_web::services::oidc::*;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::http::Cookie;
use rocket::http::Status;

mod utils;
use go_web::guards::SESSION_COOKIE;
use go_web::models::features::{Features, LoginFeature};
use rocket::tokio::sync::Mutex;
use thirtyfour::By;
use thirtyfour::WebDriver;
use utils::*;

#[test]
fn login_redirect_to_google_require_feature() {
    let (client, conn) = launch_with("");
    global_features(
        &Features {
            login: LoginFeature {
                google: false,
                ..Default::default()
            },
            ..Default::default()
        },
        &conn,
    );

    let response = client.get("/go/login/google").dispatch();

    assert_eq!(response.status(), Status::Conflict);
}

#[test]
fn login_redirect_to_google_require_to_not_be_logged_in() {
    let (client, conn) = launch_with("some_session_id: some_mail@mail.com");
    user("some_mail@mail.com", "pwd", &[], &[], &conn);
    global_features(
        &Features {
            login: LoginFeature {
                google: true,
                ..Default::default()
            },
            ..Default::default()
        },
        &conn,
    );

    let response = client
        .get("/go/login/google")
        .cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[cfg(feature = "mock")]
#[test]
fn login_redirect_to_google() {

    let mut oidc_service = OidcService::faux();


    faux::when!(oidc_service.redirect())
    .then_return(Ok(("http://auth_url".to_string(), "nonce".to_string())));


    let (client, conn) = launch_with_sessions_and_mock("", oidc_service);
    global_features(
        &Features {
            login: LoginFeature {
                google: true,
                ..Default::default()
            },
            ..Default::default()
        },
        &conn,
    );

    let response = client.get("/go/login/google").dispatch();

    assert_eq!(response.status(), Status::PermanentRedirect);

    let mut location = response.headers().get("Location");
    let url = location.next().unwrap();
    assert_eq!(url, "http://auth_url");
    assert_eq!(location.next(), None);

    assert!(response.cookies().get(SESSION_COOKIE).is_some());
}

#[test]
fn login_google_after_redirect_required_session_nonce() {
    let (client, conn) = launch_with("");
    user("some_mail@mail.com", "pwd", &[], &[], &conn);
    global_features(
        &Features {
            login: LoginFeature {
                google: true,
                ..Default::default()
            },
            ..Default::default()
        },
        &conn,
    );

    let response = client        
    .get("/go/login/redirect/google?state=4SOFn03KuR72BXkANMKnoQ&code=4%2F0AX4XfWiOsOLO15xFa1a71OykdxNOu8T-M-JaZh0dIOc3hupHDJGXUYpIx-ILk3nELtQEFw&scope=email+openid+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fuserinfo.email&authuser=1&prompt=consent").cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
    .dispatch(); 

    assert_eq!(response.status(), Status::Unauthorized);
}

#[cfg(feature = "mock")]
#[test]
fn login_google_after_redirect_for_known_user() {


    let mut oidc_service = OidcService::faux();


    faux::when!(oidc_service.retrieve_token("code", "nonce"))
        .then_return(Ok(TokenRes {mail: "some_mail@mail.com".to_string()}));


    let (client, conn) = launch_with_sessions_and_mock("some_session_id: nonce", oidc_service);

    user("some_mail@mail.com", "pwd", &[], &[], &conn);
    global_features(
        &Features {
            login: LoginFeature {
                google: true,
                ..Default::default()
            },
            ..Default::default()
        },
        &conn,
    );

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
fn login_google_after_redirect_for_unknown_user() {
    use go_web::models::users::User;

    let mut oidc_service = OidcService::faux();
    faux::when!(oidc_service.retrieve_token("code", "nonce"))
        .then_return(Ok(TokenRes {mail: "some_mail@mail.com".to_string()}));


    let (client, conn) = launch_with_sessions_and_mock("some_session_id: nonce", oidc_service);

    global_features(
        &Features {
            login: LoginFeature {
                google: true,
                ..Default::default()
            },
            ..Default::default()
        },
        &conn,
    );

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
fn login_google_after_redirect_for_unknown_user_with_default_capabilities() {
    use go_web::models::users::{User, Capability};

    let mut oidc_service = OidcService::faux();
    faux::when!(oidc_service.retrieve_token("code", "nonce"))
        .then_return(Ok(TokenRes {mail: "some_mail@mail.com".to_string()}));


    let (client, conn) = launch_with_sessions_and_mock("some_session_id: nonce", oidc_service);

    default_capabilities(&[Capability::ShortcutsWrite, Capability::TeamsWriteWithValidation],&conn);
    global_features(
        &Features {
            login: LoginFeature {
                google: true,
                ..Default::default()
            },
            ..Default::default()
        },
        &conn,
    );

    let response = client        
    .get("/go/login/redirect/google?state=4SOFn03KuR72BXkANMKnoQ&code=code&scope=email+openid+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fuserinfo.email&authuser=1&prompt=consent").cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
    .dispatch(); 

    assert_eq!(response.status(), Status::PermanentRedirect);

    let mut location = response.headers().get("Location");
    let url = location.next().unwrap();
    
    assert_eq!(url, "/");
    assert!(     location.next().is_none()); 

    assert_eq!(get_user("some_mail@mail.com", &conn), Some(User {mail: "some_mail@mail.com".to_string(), capabilities: vec![Capability::ShortcutsWrite, Capability::TeamsWriteWithValidation]}));

}

#[async_test]
async fn login_page_has_oauth_links() {
    in_browser(
        "",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let conn = con.lock().await;
                global_features(
                    &Features {
                        login: LoginFeature {
                            google: false,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    &conn,
                );

                driver
                    .get(format!("http://localhost:{}/go/login?from=allo", port))
                    .await?;

                assert!(driver
                    .find_element(By::Css("a[aria-label='Login with google']"))
                    .await
                    .is_err());

                global_features(
                    &Features {
                        login: LoginFeature {
                            google: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    &conn,
                );

                driver
                    .get(format!("http://localhost:{}/go/login?from=allo", port))
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
