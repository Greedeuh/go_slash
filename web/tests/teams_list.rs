use diesel::PgConnection;
use go_web::teams::TeamCapability;
use go_web::users::Capability;
use rocket::async_test;
use rocket::futures::FutureExt;
use rocket::tokio::sync::Mutex;
use thirtyfour::prelude::*;

mod utils;
use go_web::guards::SESSION_COOKIE;

use utils::*;

#[async_test]
async fn link_are_shown_on_other_pages() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                team("slug1", "team1", false, true, &mut con);
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("slug1", &[], 0, true)],
                    &Capability::all(),
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;

                driver.get(host(port, "")).await?;

                assert!(driver
                    .find(By::Css("a [href='/go/teams']"))
                    .await
                    .is_err());

                let endpoints = vec!["", "go/teams", "go/features", "azdaz"];

                for endpoint in endpoints {
                    driver
                        .get(format!("http://host.docker.internal:{}/{}", port, dbg!(endpoint)))
                        .await?;

                    assert_eq!(
                        driver
                            .find(By::Css("[href='/go/teams']"))
                            .await?
                            .text()
                            .await?,
                        "teams"
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
async fn with_icons() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                team("slug1", "team1", false, true, &mut con);
                team("slug2", "team2", true, true, &mut con);
                team("slug3", "team3", true, false, &mut con);
                team("slug4", "team4", false, false, &mut con);
                user("some_mail@mail.com", "pwd", &[], &[], &mut con);

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;
                let texts_sorted = ["Global", "team1", "team2", "team3", "team4"];
                let href_sorted = ["/go/teams/",
                    "/go/teams/slug1",
                    "/go/teams/slug2",
                    "/go/teams/slug3",
                    "/go/teams/slug4"];
                let locks = [false, false, true, true, false];
                let checks = [true, true, true, false, false];

                driver
                    .get(host(port, "/go/teams"))
                    .await?;

                let articles = driver.find_all(By::Css("[role='listitem']")).await?;

                for i in 0..texts_sorted.len() {
                    let article = &articles[i];
                    print!("{i}");
                    assert!(article.text().await?.starts_with(texts_sorted[i]));
                    assert_eq!(
                        article.attr("href").await?,
                        Some(href_sorted[i].to_owned())
                    );

                    println!("{i}");
                    if locks[i] {
                        article.find(By::Css(".icon-lock")).await?;
                    } else {
                        assert!(article.find(By::Css(".icon-lock")).await.is_err());
                    }
                    if checks[i] {
                        article.find(By::Css(".icon-check")).await?;
                        assert!(article
                            .find(By::Css(".icon-check-empty"))
                            .await
                            .is_err());
                    } else {
                        assert!(article.find(By::Css(".icon-check")).await.is_err());
                        article.find(By::Css(".icon-check-empty")).await?;
                    }
                }

                Ok(())
            }
            .boxed()
        },
    )
    .await;
}

#[async_test]
async fn user_team_then_others() {
    in_browser(
        "some_session_id: some_mail@mail.com",
        |driver: &WebDriver, con: Mutex<PgConnection>, port: u16| {
            async move {
                let mut con = con.lock().await;
                team("slug1", "team1", false, true, &mut con);
                team("slug2", "team2", true, true, &mut con);
                team("slug3", "team3", true, false, &mut con);
                team("slug4", "team4", false, false, &mut con);
                user(
                    "some_mail@mail.com",
                    "pwd",
                    &[("slug1", &[], 0, true)],
                    &[],
                    &mut con,
                );
                // another user should not change the behaviour
                user(
                    "another@mail.com",
                    "pwd",
                    &[
                        ("slug2", &[], 0, true),
                        ("slug3", &TeamCapability::all(), 0, true),
                    ],
                    &[],
                    &mut con,
                );

                driver
                    .add_cookie(Cookie::new(SESSION_COOKIE, "some_session_id"))
                    .await?;

                driver
                    .get(host(port, "/go/teams"))
                    .await?;

                let user_team = driver
                    .find(By::Css("[aria-label='User teams'] [role='listitem']"))
                    .await?;
                assert!(dbg!(user_team.text().await?).starts_with("team1"));

                let other_teams = driver
                    .find_all(By::Css("[aria-label='Other teams'] [role='listitem']"))
                    .await?;

                let texts_sorted = ["Global", "team2", "team3", "team4"];
                for i in 0..texts_sorted.len() {
                    let article = &other_teams[i];
                    assert!(dbg!(article.text().await)?.starts_with(dbg!(texts_sorted[i])));
                }
                Ok(())
            }
            .boxed()
        },
    )
    .await;
}
