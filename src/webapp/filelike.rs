use rocket::http::Status;
use rocket::response::{NamedFile, Responder, Response};
use rocket_contrib::Template;
use std::result;

pub enum FileLike {
    NamedFile(Option<NamedFile>),
    Template(Option<Template>),
}

impl<'r> Responder<'r> for FileLike {
    fn respond(self) -> result::Result<Response<'r>, Status> {
        match self {
            FileLike::NamedFile(file) => file.respond(),
            FileLike::Template(file) => file.respond(),
        }
    }
}
