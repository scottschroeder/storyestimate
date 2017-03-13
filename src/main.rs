#![feature(plugin)]
#![plugin(rocket_codegen)]
// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate redis;
extern crate rustc_serialize;
extern crate rand;
extern crate hyper;

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

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct APIKey {
    user_id: String,
    user_key: Option<String>,
}

impl<'a, 'r> FromRequest<'a, 'r> for APIKey {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<APIKey, ()> {
        let user_auth: String = match request.headers().get_one("Authorization") {
            Some(auth_data) => auth_data.to_string(),
            None => return Outcome::Failure((Status::Unauthorized, ())),
        };

        let base64_encoded_auth = user_auth.replace("Basic ", "");
        let authdata: Basic = match Basic::from_str(&base64_encoded_auth) {
            Ok(authdata) => authdata,
            Err(_) => return Outcome::Failure((Status::Unauthorized, ())),
        };
        //Ok(format!("{:?} -> {:?}", keys, authdata))
        Outcome::Success(APIKey {
            user_id: authdata.username,
            user_key: authdata.password,
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

use hyper::header::Basic;
use std::str::FromStr;

#[get("/test")]
fn test(keys: APIKey) -> Result<String> {
    Ok(format!("{:?}", keys))
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

#[post("/user", format = "application/json", data = "<name>")]
fn create_user(name: JSON<NameForm>,
               pool: State<RedisPool>)
               -> Result<JSON<Value>> {
    let conn = pool.get().unwrap();

    info!("Name Object: {:?}", name);
    let u = user::User::new(Some(&name.0.name));
    redisutil::save(&u, &conn)?;

    Ok(JSON(json!({
        "user_id": u.user_id,
        "user_token": u.user_token,
        "nickname": u.nickname,
    })))
}

#[post("/session/<session_id>/user/<user_id>", format = "application/json")]
fn join_session(session_id: String,
             user_id: String,
             keys: APIKey,
             pool: State<RedisPool>)
             -> Result<()> {

    let conn = pool.get().unwrap();

    let u = user::User::lookup_strict(&user_id, &conn)?;
    let s = session::Session::lookup_strict(&session_id, &conn)?;
    if u.is_authorized(&keys) {
        s.associate(&u.user_id, "user", &conn)?;
        redisutil::update_session(&session_id, &conn)?;
    } else {
        bail!(ErrorKind::UserForbidden("Caller is not authorized for this user".to_owned()))
    }
    Ok(())
}

#[patch("/session/<session_id>/user/<user_id>", format = "application/json", data = "<vote>")]
fn cast_vote(session_id: String,
             user_id: String,
             vote: JSON<VoteForm>,
             keys: APIKey,
             pool: State<RedisPool>)
             -> Result<()> {
    let conn = pool.get().unwrap();

    let mut u = user::User::lookup_strict(&user_id, &conn)?;

    if u.is_authorized(&keys) {
        u.vote(vote.0.vote);
        redisutil::save(&u, &conn)?;
        redisutil::update_session(&session_id, &conn)?;
        Ok(())
    } else {
        bail!(ErrorKind::UserForbidden("User is not authorized for this user".to_owned()))
    }
}

#[delete("/session/<session_id>/user/<user_id>")]
fn kick_user(session_id: String,
               user_id: String,
               keys: APIKey,
               pool: State<RedisPool>)
               -> Result<Option<()>> {
    let conn = pool.get().unwrap();

    let u = match user::User::lookup(&user_id, &conn)? {
        Some(u) => u,
        None => return Ok(None)
    };
    let s = session::Session::lookup_strict(&session_id, &conn)?;

    // If the caller is either the user or a session admin
    let authorized = if u.user_id == keys.user_id {
        if u.is_authorized(&keys) {
            true
        } else {
            false
        }
    } else {
        // The Caller is not the user, so check the token to be sure its valid
        if redisutil::check_token(&keys, &conn)? {
            let admins = s.get_associates("admin", &conn)?;
            if admins.contains(&keys.user_id) {
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    if authorized {
        s.disassociate(&user_id, "user", &conn)?;
        redisutil::update_session(&session_id, &conn)?;
        Ok(Some(()))
    } else {
        bail!(ErrorKind::UserForbidden("Caller is not authorized to kick user".to_owned()))
    }
}

#[delete("/user/<user_id>")]
fn delete_user(user_id: String,
               keys: APIKey,
               pool: State<RedisPool>)
               -> Result<Option<()>> {
    let conn = pool.get().unwrap();
    Ok(Some(()))

    // match user::User::lookup(&user_id, &conn)? {
    //     Some(mut u) => {
    //         if !u.is_authorized(&keys) {
    //             let session_admin = match session::Session::lookup(&session_id, &conn)? {
    //                 Some(s) => true,//s.is_authorized(&keys.session_key),
    //                 None => false,
    //             };
    //             if !session_admin {
    //                 bail!(ErrorKind::UserForbidden(
    //                     "User is not authorized for this user".to_owned()
    //                 ))
    //             }
    //         }
    //         u.delete(&conn)?;
    //         redisutil::update_session(&session_id, &conn)?;
    //         Ok(Some(()))
    //     }
    //     None => Ok(None),
    // }
}

// TODO remove the option, anyone creating a session should have a user
#[post("/session")]
fn create_session(pool: State<RedisPool>, o_keys: Option<APIKey>) -> Result<JSON<Value>> {
    let conn = pool.get().unwrap();

    let s = session::Session::new();
    if !session::session_clean(&s.session_id, &conn)? {
        // TODO: we should probably just retry a few times
        bail!("Tried to create a session with id that already exists!");
    }

    if let Some(keys) = o_keys {
        if redisutil::check_token(&keys, &conn)? {
            s.associate(&keys.user_id, "admin", &conn)?;
        }
    }

    redisutil::save(&s, &conn)?;
    Ok(JSON(json!({
        "session_id": s.session_id,
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
            //let admins = s.get_associates("admins", &conn)?;
            let admins = Vec::new();
            let public_session = session::PublicSession::new(&s, &users, &admins);
            let mut authorized = true;//s.is_authorized(&keys.session_key);
            if !authorized {
                for u in users {
                    if u.is_authorized(&keys) {
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
            if true {// !s.is_authorized(&keys.session_key) {
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
            if true {// !s.is_authorized(&keys.session_key) {
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
            test,
            files,
            create_user,
            join_session,
            cast_vote,
            kick_user,
            delete_user,
            create_session,
            update_session,
            lookup_session,
            delete_session,
        ])
        .manage(pool)
        .launch();
}
