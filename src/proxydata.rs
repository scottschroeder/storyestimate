use rocket::request::{self, Request, FromRequest};
use rocket::{self, Outcome};
use rocket::http::Status;

#[derive(Debug)]
pub struct ProxyData {
    pub http_host: String,
    pub scheme: String,
}

impl<'a, 'r> FromRequest<'a, 'r> for ProxyData {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<ProxyData, ()> {
        let config = match rocket::config::active() {
            Some(config) => config,
            None => {
                warn!("Could not load rocket config while processing ProxyData");
                return Outcome::Failure((Status::InternalServerError, ()));
            }
        };
        let http_host: String = match request.headers().get_one("Host") {
            Some(host) => host.to_string(),
            None => format!("{}:{}", config.address, config.port),
        };
        let http_scheme: String = match request.headers().get_one("Scheme") {
            Some(scheme) => scheme.to_string(),
            None => "http".to_string(),
        };

        Outcome::Success(ProxyData {
            http_host: http_host,
            scheme: http_scheme,
        })
    }
}
