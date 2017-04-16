use hyper::error::Error as HyperError;
use redis::{self, RedisError, Value};
use rocket::config::ConfigError;
use serde_json;
use std::io;


impl From<Error> for RedisError {
    fn from(e: Error) -> RedisError {
        (redis::ErrorKind::TypeError, "storyestimate error", e.description().to_owned()).into()
    }
}

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Parse(serde_json::Error);
        RedisError(RedisError);
        IOError(io::Error);
        HyperError(HyperError);
        RocketConfigError(ConfigError);
    }
    errors {
        // Web Errors
        ResourceNotFound(t: String) {
            description("Could not find resource at URL")
                display("unrecognized: '{}'", t)
        }
        UserError(t: String) {
            description("User attempted invalid operation")
                display("{}", t)
        }
        UserUnauthorized {
            description("User could not be authenticated")
                display("User could not be authenticated")
        }

        // StoryEstimates Errors
        UserAlreadyExists {
            description("The user_id already exists")
                display("The user_id already exists")
        }
        ParticipantNameExists {
            description("The chosen nickname is already taken")
                display("The chosen nickname is already taken")
        }
        ObjectNotFound(t: String) {
            description("Could not find object in data backend")
                display("missing: '{}'", t)
        }
        UserForbidden(t: String) {
            description("User attempted operation without proper credentials")
                display("{}", t)
        }

        // Data Backend Errors
        DataIntegrityError(t: String) {
            description("The internal data store is inconsistent")
                display("{}", t)
        }
        UnexpectedRedisResponse(v: Value) {
            description("Got an unexpected response from redis")
                display("{:?}", v)
        }
    }
}
