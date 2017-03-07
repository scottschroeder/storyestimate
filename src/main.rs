#![feature(plugin)]
#![plugin(rocket_codegen)]
// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate redis;
extern crate rustc_serialize;
extern crate rand;

//use redis::{Client, Commands, Connection, RedisError, RedisResult, Value};
use redis::{Client, Commands};

use rustc_serialize::json;

use std::convert::From;


extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;

use std::path::PathBuf;
use rocket::response::NamedFile;
use rocket_contrib::{JSON, Value};
use std::env;
use std::{thread, time};

mod user;
mod session;
mod errors;
mod generator;
mod redisutil;

use errors::*;
use redisutil::RedisBackend;


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

// For testing, and making DDOS really easy
#[get("/sleep/<seconds>")]
fn sleep(seconds: u64) -> String {
    let sleep_time = time::Duration::from_secs(seconds);
    thread::sleep(sleep_time);
    format!("Slept for {} seconds.", seconds)
}

#[get("/static/<file..>")]
fn files(file: PathBuf) -> Result<Option<NamedFile>> {
    let fullpath = env::current_dir()?
        .join("static/")
        .join(file);
    info!("Full Path is: {:?}", fullpath.display());
    Ok(NamedFile::open(fullpath).ok())
}

#[post("/session")]
fn create_session() -> Result<JSON<Value>> {
    let client = Client::open("redis://127.0.0.1/")?;
    let conn = client.get_connection()?;

    let s = session::Session::new();
    if s.exists(&conn)? {
        bail!("Tried to create a session with id that already exists!");
    }
    let _: () = conn.set(s.unique_key(), &s)?;
    Ok(JSON(json!({
        "session_id": s.session_id,
        "session_admin_token": s.session_admin_token,
    })))
}

#[get("/session/<session_id>")]
fn lookup_session(session_id: String) -> Result<Option<JSON<Value>>> {
    let client = Client::open("redis://127.0.0.1/")?;
    let conn = client.get_connection()?;

    let possible_session = session::Session::lookup(session_id, &conn)?;

    match possible_session {
        Some(s) => Ok(Some(JSON(json!({
                "session_id": s.session_id,
                "average": s.average,
        })))),
        None => Ok(None)
    }

    // Ok(JSON(json!({
    //     "session_id": s.session_id,
    //     "session_admin_token": s.session_admin_token,
    // })))
}

fn main() {

    // generator::show_string();
    // return;
    rocket::ignite()
        .mount("/", routes![
            index,
            files,
            create_session,
            lookup_session,
            sleep,
        ]).launch();
}
