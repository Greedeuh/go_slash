use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{MediaType, Status},
    request::Outcome,
    response::Responder,
    Request, Response, State,
};
use rocket_dyn_templates::Template;
use serde_json::json;

use crate::AppConfig;

pub struct UnauthorizedAsLogin;

#[rocket::async_trait]
impl Fairing for UnauthorizedAsLogin {
    // This is a request and response fairing named "GET/POST Counter".
    fn info(&self) -> Info {
        Info {
            name: "UnauthorizedAsLogin",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        if let Some(accept) = request.accept() && !accept.media_types().any(|x| x ==&MediaType::HTML) {
            return;
        }
        if response.status() != Status::Unauthorized {
            return;
        }

        info!("UnauthorizedAsLogin processing");

        let conf = match request.guard::<&State<AppConfig>>().await {
            Outcome::Success(x) => x,
            Outcome::Failure(e) => {
                error!(
                    "UnauthorizedAsLogin fail getting State<AppConfig>: Failure {:?}",
                    e
                );
                return;
            }
            Outcome::Forward(e) => {
                error!(
                    "UnauthorizedAsLogin fail getting State<AppConfig>: Forward {:?}",
                    e
                );
                return;
            }
        };

        let template = Template::render(
            "login",
            json!({ "context": json!({ "simple_salt": conf.simple_login_salt1 }).to_string() }),
        );

        match template.respond_to(request) {
            Ok(res) => response.merge(res),
            e => error!(
                "UnauthorizedAsLogin fail getting a response template: {:?}",
                e
            ),
        }
    }
}
