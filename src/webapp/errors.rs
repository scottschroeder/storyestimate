

use errors::*;
use rocket;
use rocket::Catcher;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};


use serde_json;
use std::io::Cursor;
use std::result;

impl<'r> Responder<'r> for Error {
    fn respond(self) -> result::Result<Response<'r>, Status> {
        info!("This is a: {:?}", self);
        let (message, status) = match self {
            Error(ErrorKind::ObjectNotFound(reason), _) => (reason, Status::NotFound),
            Error(ErrorKind::ResourceNotFound(reason), _) => (reason, Status::NotFound),
            Error(ErrorKind::UserError(reason), _) => (reason, Status::BadRequest),
            Error(ErrorKind::UserForbidden(reason), _) => (reason, Status::Forbidden),
            Error(ErrorKind::UserUnauthorized, _) => {
                ("Unauthorized".to_string(), Status::Unauthorized)
            },
            _ => (format!("{}", self), Status::InternalServerError),
            //Error(err, _) => (err.display(), Status::InternalServerError),
        };
        warn!("Returning {:?}: {:?}", status, message);

        let body = json!({
            "error": message,
            "status": status.code,
        });
        Response::build()
            .sized_body(Cursor::new(serde_json::to_vec(&body).unwrap()))
            .status(status)
            .ok()
    }
}

pub fn errors() -> Vec<Catcher> {
    errors![unauthorized, not_found]
}

#[error(401)]
fn unauthorized(_req: &Request) -> Result<()> {
    Err(ErrorKind::UserUnauthorized.into())
}

#[error(404)]
fn not_found(req: &Request) -> Result<()> {
    Err(ErrorKind::ResourceNotFound(String::from(req.uri().as_str())).into())
}
