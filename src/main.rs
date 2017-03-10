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
use rocket::http::Status;

#[macro_use]
extern crate serde_derive;
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
struct SessionStateForm {
    state: session::SessionState,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
struct VoteForm {
    vote: u32,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
struct NameForm {
    name: String,
}

use rocket::response::{Response, Responder};

impl<'r> Responder<'r> for Error {
    fn respond(self) -> std::result::Result<Response<'r>, Status> {
        info!("This is a: {:?}", self);
        let (message, status) = match self {
            Error(ErrorKind::UserError(reason), _) => (reason, Status::BadRequest),
            _ => (format!("{}", self), Status::InternalServerError),
            //Error(err, _) => (err.display(), Status::InternalServerError),
        };
        warn!("Returning {:?}: {:?}", status, message);
        Err(status)
    }
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
    let fullpath = env::current_dir()
        ?
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
                bail!(ErrorKind::UserError("Tried to create a user with name that already \
                                            exists!"
                    .to_owned()));
            }
            let _: () = conn.set(u.unique_key(), &u)?;
            Ok(JSON(json!({
                "user_token": u.user_token,
                "session_id": u.session_id,
                "nickname": u.nickname,
            })))
        }
        None => {
            bail!(ErrorKind::UserError(
                "Tried to create a user for a session that does not exist!".to_owned()
            ))
        }
    }
}

#[patch("/session/<session_id>/user/<name>", format = "application/json", data = "<vote>")]
fn cast_vote(session_id: String, name: String, vote: JSON<VoteForm>) -> Result<()> {
    let client = Client::open("redis://127.0.0.1/")?;
    let conn = client.get_connection()?;

    let user_id = format!("{}_{}", session_id, name);
    let possible_user = user::User::lookup(&user_id, &conn)?;
    match possible_user {
        Some(mut u) => {
            u.vote(vote.0.vote);
            let _: () = conn.set(u.unique_key(), &u)?;
        }
        None => {
            bail!(ErrorKind::UserError("Tried to cast a vote for a non-existent user!".to_owned()))
        }
    };
    Ok(())
}

#[delete("/session/<session_id>/user/<name>")]
fn delete_user(session_id: String, name: String) -> Result<Option<()>> {
    let client = Client::open("redis://127.0.0.1/")?;
    let conn = client.get_connection()?;

    let user_id = format!("{}_{}", session_id, name);
    let possible_user = user::User::lookup(&user_id, &conn)?;
    match possible_user {
        Some(mut u) => {
            u.delete(&conn)?;
            Ok(Some(()))
        }
        None => Ok(None),
    }
}

#[post("/session")]
fn create_session() -> Result<JSON<Value>> {
    let client = Client::open("redis://127.0.0.1/")?;
    let conn = client.get_connection()?;

    let s = session::Session::new();
    if session::session_clean(&s.session_id, &conn)? {
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
        }
        None => Ok(None),
    }
}

#[patch("/session/<session_id>", format = "application/json", data = "<state>")]
fn update_session(session_id: String, state: JSON<SessionStateForm>) -> Result<()> {
    let client = Client::open("redis://127.0.0.1/")?;
    let conn = client.get_connection()?;

    let possible_session = session::Session::lookup(&session_id, &conn)?;

    match possible_session {
        Some(mut s) => {
            let query_string = format!("{}_*", s.session_id);
            let mut users = user::User::bulk_lookup(&query_string, &conn)?;
            match state.0.state {
                session::SessionState::Reset => s.reset(&mut users),
                session::SessionState::Vote => s.clear(&mut users),
                session::SessionState::Visible => s.take_votes(&mut users),
                //TODO: This should be some 4xx error, maybe the same one if it couldn't be decoded
                // Alternately, maybe there's some other task this can do?
                session::SessionState::Dirty => {
                    bail!(ErrorKind::UserError("Tried to set the state to 'Dirty'".to_owned()))
                }
            }
            for u in users {
                let _: () = conn.set(u.unique_key(), &u)?;
            }
            let _: () = conn.set(s.unique_key(), &s)?;
        }
        None => bail!(ErrorKind::UserError("Tried to update a non-existent session!".to_owned())),
    }
    Ok(())
}

#[delete("/session/<session_id>")]
fn delete_session(session_id: String) -> Result<Option<()>> {
    let client = Client::open("redis://127.0.0.1/")?;
    let conn = client.get_connection()?;

    let possible_session = session::Session::lookup(&session_id, &conn)?;

    let query_string = format!("{}_*", session_id);
    let users = user::User::bulk_lookup(&query_string, &conn)?;
    for mut u in users {
        u.delete(&conn)?
    }

    match possible_session {
        Some(mut s) => {
            s.delete(&conn)?;
            Ok(Some(()))
        }
        None => Ok(None),
    }
}

fn main() {
    rocket::ignite()
        .mount("/",
               routes![
            index,
            files,
            create_user,
            cast_vote,
            delete_user,
            create_session,
            update_session,
            lookup_session,
            delete_session,
            sleep,
        ])
        .launch();
}
