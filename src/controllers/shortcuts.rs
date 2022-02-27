use log::error;
use std::path::PathBuf;

use rocket::{http::Status, response::Redirect, State};

use crate::Entries;

#[get("/<shortcut..>")]
pub fn shortcuts(shortcut: PathBuf, entries: &State<Entries>) -> Result<Redirect, Status> {
    let shortcut = match shortcut.to_str() {
        Some(shortcut) => shortcut,
        None => {
            error!("GET <shortcut..> failed parsing: {:?}", shortcut);
            return Err(Status::BadRequest);
        }
    };

    match entries.find(shortcut) {
        Some(url) => Ok(Redirect::permanent(url)),
        None => Err(Status::NotFound),
    }
}
