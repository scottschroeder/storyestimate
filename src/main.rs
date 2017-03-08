#![feature(plugin)]
#![plugin(rocket_codegen)]
// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate redis;
extern crate rustc_serialize;
extern crate rand;

//use redis::{Client, Commands, Connection, RedisError, RedisResult, Value};
use redis::{Client, Commands};

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
#[macro_use] extern crate serde_derive;
use std::env;
use std::{thread, time};

mod user;
mod session;
mod errors;
mod generator;
mod redisutil;

use errors::*;
use redisutil::RedisBackend;

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
struct VoteForm {
    vote: u32
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
struct NameForm {
    name: String
}

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

#[post("/session/<session_id>/user", format = "application/json", data = "<name>")]
fn create_user(session_id: String, name: JSON<NameForm>) -> Result<JSON<Value>> {
    let client = Client::open("redis://127.0.0.1/")?;
    let conn = client.get_connection()?;

    let possible_session = session::Session::lookup(&session_id, &conn)?;

    info!("Name Object: {:?}", name);
    match possible_session {
        Some(_) => {
            let u = user::User::new(&session_id, &name.0.name);
            if u.exists(&conn)? {
                // TODO: This should be a 4xx
                bail!("Tried to create a user with name that already exists!");
            }
            let _: () = conn.set(u.unique_key(), &u)?;
            Ok(JSON(json!({
                "user_token": u.user_token,
                "session_id": u.session_id,
                "nickname": u.nickname,
            })))
        },
        // TODO: should be 4xx or maybe 404?
        None => bail!("Tried to create a user for a session that does not exist!")
    }
}

//TODO: Should be a PATCH
#[post("/session/<session_id>/user/<name>", format = "application/json", data = "<vote>")]
fn cast_vote(session_id: String, name: String, vote: JSON<VoteForm>) -> Result<()> {
    let client = Client::open("redis://127.0.0.1/")?;
    let conn = client.get_connection()?;

    let possible_session = session::Session::lookup(&session_id, &conn)?;

    info!("Name Object: {:?}", name);
    match possible_session {
        Some(_) => {
            let user_id = format!("{}_{}", session_id, name);
            let possible_user = user::User::lookup(&user_id, &conn)?;
            match possible_user {
                Some(mut u) => {
                    u.vote(vote.0.vote);
                    let _: () = conn.set(u.unique_key(), &u)?;
                },
                None => bail!("Tried to cast a vote for a non-existent user!"),
            };
            Ok(())
        },
        // TODO: should be 4xx or maybe 404?
        None => bail!("Tried to cast a vote in a non-existent session!")
    }
}

#[post("/session")]
fn create_session() -> Result<JSON<Value>> {
    let client = Client::open("redis://127.0.0.1/")?;
    let conn = client.get_connection()?;

    let s = session::Session::new();
    if s.exists(&conn)? {
        // TODO: we should probably just retry a few times
        bail!("Tried to create a session with id that already exists!");
    }
    let _: () = conn.set(s.unique_key(), &s)?;
    Ok(JSON(json!({
        "session_id": s.session_id,
        "session_admin_token": s.session_admin_token,
    })))
}

#[get("/session/<session_id>")]
fn lookup_session(session_id: String) -> Result<Option<JSON<session::PublicSession>>> {
    let client = Client::open("redis://127.0.0.1/")?;
    let conn = client.get_connection()?;

    let possible_session = session::Session::lookup(&session_id, &conn)?;

    match possible_session {
        Some(s) => {
            let query_string = format!("{}_*", s.session_id);
            let users = user::User::bulk_lookup(&query_string, &conn)?;
            let public_session = session::PublicSession::new(&s, &users);
            Ok(Some(JSON(public_session)))
        },
        None => Ok(None)
    }
}

fn main() {

    // generator::show_string();
    // return;
    rocket::ignite()
        .mount("/", routes![
            index,
            files,
            create_user,
            cast_vote,
            create_session,
            lookup_session,
            sleep,
        ]).launch();
}
