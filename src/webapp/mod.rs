#![allow(unmounted_route)]
use dal;
use errors::*;
use rocket;
use rocket::http::Method;
use std::path::PathBuf;

// Modules containing HTTP helpers
mod apikey;
mod proxydata;
mod filelike;
mod errors;
mod cors;
mod assumejson;

// Modules containing endpoints
mod estimates;
mod swagger;


const WELCOME_TEXT: &'static str = "Welcome to the StoryEstimates WebApp!";

#[get("/")]
fn hello() -> &'static str {
    WELCOME_TEXT
}

#[route(OPTIONS, "/<any_url..>")]
#[allow(unused_variables)] //This is so we match OPTIONS on everything
fn cors_preflight(any_url: PathBuf) -> self::cors::PreflightCORS {
    self::cors::CORS::preflight("*")
        .methods(&vec![Method::Options,
                       Method::Get,
                       Method::Post,
                       Method::Put,
                       Method::Delete,
                       Method::Head,
                       Method::Trace,
                       Method::Connect,
                       Method::Patch])
        .headers(&vec!["Content-Type",
                       "Origin",
                       "Accept",
                       "Authorization",
                       "X-Requested-With",
                       "X-API-Key"])
}

pub fn build_webapp<P>(storydata_provider: P) -> rocket::Rocket
    where P: 'static,
          P: dal::StoryDataProvider
{
    rocket::ignite()
        .mount("/", routes![hello, cors_preflight])
        .mount("/api", self::estimates::routes())
        .mount("/docs", self::swagger::routes())
        .catch(self::errors::errors())
        .manage(storydata_provider)
}

#[cfg(test)]
mod test {
    use dal::SharedMemoryDB;
    use rocket::http::{Method, Status};
    use rocket::testing::MockRequest;

    #[test]
    fn hello_world() {
        let mem_data = SharedMemoryDB::new();
        let rocket = super::build_webapp(mem_data);
        let mut req = MockRequest::new(Method::Get, "/");
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);

        let body_str = response.body().and_then(|b| b.into_string());
        assert_eq!(body_str, Some(super::WELCOME_TEXT.to_string()));
    }

}
