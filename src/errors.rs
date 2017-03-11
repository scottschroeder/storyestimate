use rustc_serialize;
use redis::{self, RedisError};
use std::io;


impl From<Error> for RedisError {
    fn from(e: Error) -> RedisError {
        (redis::ErrorKind::TypeError, "storyestimate error", e.description().to_owned()).into()
    }
}

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Parse(rustc_serialize::json::DecoderError);
        RedisError(RedisError);
        IOError(io::Error);
    }
    errors {
        RedisEmptyError(t: String) {
            description("Could not find key in Redis")
                display("missing key: '{}'", t)
        }
        UserError(t: String) {
            description("User attempted invalid operation")
                display("{}", t)
        }
        UserForbidden(t: String) {
            description("User attempted operation without proper credentials")
                display("{}", t)
        }
    }
}
