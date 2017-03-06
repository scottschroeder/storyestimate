use rustc_serialize;
use redis::{self, RedisError};


impl From<Error> for RedisError {
    fn from(e: Error) -> RedisError {
        (redis::ErrorKind::TypeError, "storyestimate error", e.description().to_owned()).into()
    }
}

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Parse(rustc_serialize::json::DecoderError);
    }
}
