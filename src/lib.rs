#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    ""
}

pub fn server() -> rocket::Rocket<rocket::Build> {
    rocket::build().mount("/", routes![index])
}
