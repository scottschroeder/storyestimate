#![feature(plugin)]
#![plugin(rocket_codegen)]
// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate redis;
extern crate rustc_serialize;

//use redis::{Client, Commands, Connection, RedisError, RedisResult, Value};
//use redis::{ToRedisArgs, FromRedisValue};

use rustc_serialize::json;

use std::convert::From;


extern crate rocket;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;

use std::path::{PathBuf};
use rocket::response::NamedFile;
use std::env;
use std::{thread, time};

mod user;
mod session;
mod errors;

use errors::*;


#[get("/")]
fn index() -> &'static str {
    debug!("Debug Log");
    info!("Info Log");
    warn!("Warning Log");
    error!("Error Log");
    "Hello, world!"
}

#[get("/cwd")]
fn current_dir() -> String {
    env::current_dir().unwrap().to_str().unwrap().to_owned()
}


#[get("/sleep/<seconds>")]
fn sleep(seconds: u64) -> String {
    let sleep_time = time::Duration::from_secs(seconds);
    thread::sleep(sleep_time);
    format!("Slept for {} seconds.", seconds)
}

#[get("/file/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    //TODO: Replace unwrap with option
    let fullpath = env::current_dir().unwrap().join("static/").join(file);
    //let fullpath = Path::new("static").join(file);
    info!("Full Path is: {:?}", fullpath.display());
    NamedFile::open(fullpath).ok()
}

fn main() {
    rocket::ignite().mount("/", routes![
        index,
        files,
        current_dir,
        sleep,
    ]).launch();
}
