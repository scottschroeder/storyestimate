#![feature(plugin)]
#![plugin(rocket_codegen)]
// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate redis;
extern crate rustc_serialize;
extern crate rand;

//use redis::{Client, Commands, Connection, RedisError, RedisResult, Value};
use redis::{ConnectionInfo, ConnectionAddr};

use std::convert::From;


extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;
extern crate r2d2;
extern crate r2d2_redis;
extern crate num_cpus;

use std::path::PathBuf;
use rocket::response::NamedFile;
use rocket_contrib::{JSON, Value};
use rocket::http::Status;
use rocket::{Outcome, State};
use rocket::request::{self, Request, FromRequest};

use r2d2_redis::RedisConnectionManager;
type RedisPool = r2d2::Pool<r2d2_redis::RedisConnectionManager>;

#[macro_use]
extern crate serde_derive;
use std::env;
use std::{thread, time};

mod auth;
mod user;
mod session;
mod errors;
mod generator;
mod redisutil;

use errors::*;
use redisutil::RedisBackend;
use auth::ObjectAuth;


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

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
struct APIKey {
    session_key: Option<String>,
    user_key: Option<String>,
}

enum GetOneOptions<T> {
    None,
    One(T),
    Many,
}

fn get_only_one<T>(mut source_list: Vec<T>) -> GetOneOptions<T> {
    let possible_item = source_list.pop();
    if !source_list.is_empty() {
        return GetOneOptions::Many;
    }
    match possible_item {
        None => GetOneOptions::None,
        Some(x) => GetOneOptions::One(x),
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for APIKey {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<APIKey, ()> {
        let session_keys: Vec<_> = request.headers().get("session-token").collect();
        let session_key: Option<String> = match get_only_one(session_keys) {
            GetOneOptions::None => None,
            GetOneOptions::One(key) => Some(key.to_string()),
            GetOneOptions::Many => return Outcome::Failure((Status::BadRequest, ())),
        };
        let user_keys: Vec<_> = request.headers().get("user-token").collect();
        let user_key: Option<String> = match get_only_one(user_keys) {
            GetOneOptions::None => None,
            GetOneOptions::One(key) => Some(key.to_string()),
            GetOneOptions::Many => return Outcome::Failure((Status::BadRequest, ())),
        };

        Outcome::Success(APIKey {
            session_key: session_key,
            user_key: user_key,
        })
    }
}

use rocket::response::{Response, Responder};

impl<'r> Responder<'r> for Error {
    fn respond(self) -> std::result::Result<Response<'r>, Status> {
        info!("This is a: {:?}", self);
        let (message, status) = match self {
            Error(ErrorKind::UserError(reason), _) => (reason, Status::BadRequest),
            Error(ErrorKind::UserForbidden(reason), _) => (reason, Status::Forbidden),
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
fn create_user(session_id: String,
               name: JSON<NameForm>,
               pool: State<RedisPool>)
               -> Result<JSON<Value>> {
    let conn = pool.get().unwrap();

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
            redisutil::save(&u, &conn)?;
            redisutil::update_session(&session_id, &conn)?;

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
fn cast_vote(session_id: String,
             name: String,
             vote: JSON<VoteForm>,
             keys: APIKey,
             pool: State<RedisPool>)
             -> Result<()> {
    let conn = pool.get().unwrap();

    let user_id = format!("{}_{}", session_id, name);
    let possible_user = user::User::lookup(&user_id, &conn)?;
    match possible_user {
        Some(mut u) => {
            if u.is_authorized(&keys.user_key) {
                u.vote(vote.0.vote);
                redisutil::save(&u, &conn)?;
                redisutil::update_session(&session_id, &conn)?;
            } else {
                bail!(ErrorKind::UserForbidden("User is not authorized for this user".to_owned()))
            }
        }
        None => {
            bail!(ErrorKind::UserError("Tried to cast a vote for a non-existent user!".to_owned()))
        }
    };
    Ok(())
}

#[delete("/session/<session_id>/user/<name>")]
fn delete_user(session_id: String,
               name: String,
               keys: APIKey,
               pool: State<RedisPool>)
               -> Result<Option<()>> {
    let conn = pool.get().unwrap();

    let user_id = format!("{}_{}", session_id, name);
    match user::User::lookup(&user_id, &conn)? {
        Some(mut u) => {
            if !u.is_authorized(&keys.user_key) {
                let session_admin = match session::Session::lookup(&session_id, &conn)? {
                    Some(s) => s.is_authorized(&keys.session_key),
                    None => false,
                };
                if !session_admin {
                    bail!(ErrorKind::UserForbidden(
                        "User is not authorized for this user".to_owned()
                    ))
                }
            }
            u.delete(&conn)?;
            redisutil::update_session(&session_id, &conn)?;
            Ok(Some(()))
        }
        None => Ok(None),
    }
}

#[post("/session")]
fn create_session(pool: State<RedisPool>) -> Result<JSON<Value>> {
    let conn = pool.get().unwrap();

    let s = session::Session::new();
    if session::session_clean(&s.session_id, &conn)? {
        // TODO: we should probably just retry a few times
        bail!("Tried to create a session with id that already exists!");
    }
    redisutil::save(&s, &conn)?;
    Ok(JSON(json!({
        "session_id": s.session_id,
        "session_admin_token": s.session_admin_token,
    })))
}

#[get("/session/<session_id>")]
fn lookup_session(session_id: String,
                  keys: APIKey,
                  pool: State<RedisPool>)
                  -> Result<Option<JSON<session::PublicSession>>> {
    let conn = pool.get().unwrap();

    let possible_session = session::Session::lookup(&session_id, &conn)?;

    match possible_session {
        Some(s) => {
            let query_string = format!("{}_*", s.session_id);
            let users = user::User::bulk_lookup(&query_string, &conn)?;
            let public_session = session::PublicSession::new(&s, &users);
            let mut authorized = s.is_authorized(&keys.session_key);
            if !authorized {
                for u in users {
                    if u.is_authorized(&keys.user_key) {
                        authorized = true;
                        break;
                    }
                }
            }
            if authorized {
                Ok(Some(JSON(public_session)))
            } else {
                bail!(ErrorKind::UserForbidden("User is not authorized for this user".to_owned()))
            }
        }
        None => Ok(None),
    }
}

#[patch("/session/<session_id>", format = "application/json", data = "<state>")]
fn update_session(session_id: String,
                  state: JSON<SessionStateForm>,
                  keys: APIKey,
                  pool: State<RedisPool>)
                  -> Result<()> {
    let conn = pool.get().unwrap();

    match session::Session::lookup(&session_id, &conn)? {
        Some(mut s) => {
            if !s.is_authorized(&keys.session_key) {
                bail!(ErrorKind::UserForbidden(
                    "User is not authorized as session admin".to_owned()
                ))
            }
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
                redisutil::save(&u, &conn)?;
            }
            redisutil::save(&s, &conn)?;
            redisutil::update_session(&session_id, &conn)?;
        }
        None => bail!(ErrorKind::UserError("Tried to update a non-existent session!".to_owned())),
    }
    Ok(())
}

#[delete("/session/<session_id>")]
fn delete_session(session_id: String, keys: APIKey, pool: State<RedisPool>) -> Result<Option<()>> {
    let conn = pool.get().unwrap();

    let possible_session = session::Session::lookup(&session_id, &conn)?;

    let query_string = format!("{}_*", session_id);
    let users = user::User::bulk_lookup(&query_string, &conn)?;

    match possible_session {
        Some(mut s) => {
            if !s.is_authorized(&keys.session_key) {
                bail!(ErrorKind::UserForbidden(
                    "User is not authorized as session admin".to_owned()
                ))
            }
            for mut u in users {
                u.delete(&conn)?
            }
            s.delete(&conn)?;
            redisutil::update_session(&session_id, &conn)?;
            Ok(Some(()))
        }
        None => Ok(None),
    }
}

fn main() {
    let cpus = num_cpus::get() as u32;
    let redis_ctx = ConnectionInfo {
        addr: Box::new(ConnectionAddr::Tcp("127.0.0.1".to_owned(), 6379)),
        db: 0,
        passwd: None,
    };
    //info!("Creating Redis Pool ({}x -> {:?})", cpus, redis_ctx);
    let config = r2d2::Config::builder()
        .pool_size(cpus)
        .build();
    let manager = RedisConnectionManager::new(redis_ctx).unwrap();
    let pool = r2d2::Pool::new(config, manager).unwrap();

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
        .manage(pool)
        .launch();
}
