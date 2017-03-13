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

use hyper::header::Basic;
use std::str::FromStr;
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

#[get("/<file..>")]
fn files(file: PathBuf) -> Result<Option<NamedFile>> {
    let fullpath = env::current_dir()
        ?
        .join("static/")
        .join(file);
    info!("Full Path is: {:?}", fullpath.display());
    Ok(NamedFile::open(fullpath).ok())
}

#[post("/user", format = "application/json", data = "<name>")]
fn create_user(name: JSON<NameForm>, pool: State<RedisPool>) -> Result<JSON<Value>> {
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


#[patch("/session/<session_id>/user/<user_id>", format = "application/json", data = "<vote>")]
fn cast_vote(session_id: String,
             user_id: String,
             vote: JSON<VoteForm>,
             keys: APIKey,
             pool: State<RedisPool>)
             -> Result<Option<()>> {
    let conn = pool.get().unwrap();

    let mut u = match user::User::lookup(&user_id, &conn)? {
        Some(u) => u,
        None => return Ok(None),
    };

    if u.is_authorized(&keys) {
        u.vote(vote.0.vote);
        redisutil::save(&u, &conn)?;
        redisutil::update_session(&session_id, &conn)?;
        Ok(Some(()))
    } else {
        bail!(ErrorKind::UserForbidden("User is not authorized for this user".to_owned()))
    }
}

#[delete("/user/<user_id>")]
fn delete_user(user_id: String, keys: APIKey, pool: State<RedisPool>) -> Result<Option<()>> {
    let conn = pool.get().unwrap();
    let mut u = match user::User::lookup(&user_id, &conn)? {
        Some(u) => u,
        None => return Ok(None),
    };

    if u.is_authorized(&keys) {
        u.delete(&conn)?;
        Ok(Some(()))
    } else {
        bail!(ErrorKind::UserForbidden("Caller is not authorized to delete user".to_owned()))
    }
}

#[post("/session")]
fn create_session(pool: State<RedisPool>, keys: APIKey) -> Result<JSON<Value>> {
    let conn = pool.get().unwrap();

    let s = session::Session::new();
    if !session::session_clean(&s.session_id, &conn)? {
        // TODO: we should probably just retry a few times
        bail!("Tried to create a session with id that already exists!");
    }

    if !redisutil::check_token(&keys, &conn)? {
        bail!(ErrorKind::UserForbidden("Token invalid".to_owned()))
    }

    s.associate(&keys.user_id, "admin", &conn)?;
    redisutil::save(&s, &conn)?;
    Ok(JSON(json!({
        "session_id": s.session_id,
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

#[delete("/session/<session_id>/user/<user_id>")]
fn kick_user(session_id: String,
             user_id: String,
             keys: APIKey,
             pool: State<RedisPool>)
             -> Result<Option<()>> {
    let conn = pool.get().unwrap();

    let u = match user::User::lookup(&user_id, &conn)? {
        Some(u) => u,
        None => return Ok(None),
    };
    let s = session::Session::lookup_strict(&session_id, &conn)?;

    if u.is_authorized(&keys) || is_admin(&keys, &s, &conn)? {
        s.disassociate(&user_id, "user", &conn)?;
        redisutil::update_session(&session_id, &conn)?;
        Ok(Some(()))
    } else {
        bail!(ErrorKind::UserForbidden("Caller is not authorized to kick user".to_owned()))
    }
}

#[post("/session/<session_id>/admin/<user_id>", format = "application/json")]
fn grant_admin(session_id: String,
               user_id: String,
               keys: APIKey,
               pool: State<RedisPool>)
               -> Result<Option<()>> {

    let conn = pool.get().unwrap();

    let s = match session::Session::lookup(&session_id, &conn)? {
        Some(s) => s,
        None => return Ok(None),
    };
    if is_admin(&keys, &s, &conn)? {
        s.associate(&user_id, "admin", &conn)?;
        redisutil::update_session(&session_id, &conn)?;
    } else {
        bail!(ErrorKind::UserForbidden(
            "User is not authorized as session admin".to_owned()
        ))
    }
    Ok(Some(()))
}

#[delete("/session/<session_id>/admin/<user_id>")]
fn revoke_admin(session_id: String,
                user_id: String,
                keys: APIKey,
                pool: State<RedisPool>)
                -> Result<Option<()>> {
    let conn = pool.get().unwrap();

    let s = match session::Session::lookup(&session_id, &conn)? {
        Some(s) => s,
        None => return Ok(None),
    };

    if is_admin(&keys, &s, &conn)? {
        s.disassociate(&user_id, "admin", &conn)?;
        redisutil::update_session(&session_id, &conn)?;
        Ok(Some(()))
    } else {
        bail!(ErrorKind::UserForbidden(
            "User is not authorized as session admin".to_owned()
        ))
    }
}

#[get("/session/<session_id>")]
fn lookup_session(session_id: String,
                  pool: State<RedisPool>)
                  -> Result<Option<JSON<session::PublicSession>>> {
    let conn = pool.get().unwrap();

    let s = match session::Session::lookup(&session_id, &conn)? {
        Some(s) => s,
        None => return Ok(None),
    };

    let admin_ids = s.get_associates("admin", &conn)?;
    let users = lookup_active_users(&s, &conn)?;
    let public_session = session::PublicSession::new(&s, &users, &admin_ids);
    Ok(Some(JSON(public_session)))
}

#[patch("/session/<session_id>", format = "application/json", data = "<state>")]
fn update_session(session_id: String,
                  state: JSON<SessionStateForm>,
                  keys: APIKey,
                  pool: State<RedisPool>)
                  -> Result<Option<()>> {
    let conn = pool.get().unwrap();

    let mut s = match session::Session::lookup(&session_id, &conn)? {
        Some(s) => s,
        None => return Ok(None),
    };

    if !is_admin(&keys, &s, &conn)? {
        bail!(ErrorKind::UserForbidden(
            "User is not authorized as session admin".to_owned()
        ))
    }

    let mut users = lookup_active_users(&s, &conn)?;
    match state.0.state {
        session::SessionState::Reset => s.reset(&mut users),
        session::SessionState::Vote => s.clear(&mut users),
        session::SessionState::Visible => s.take_votes(&mut users),
        session::SessionState::Dirty => {
            bail!(ErrorKind::UserError("Tried to set the state to 'Dirty'".to_owned()))
        }
    }
    for u in users {
        redisutil::save(&u, &conn)?;
    }
    redisutil::save(&s, &conn)?;
    redisutil::update_session(&session_id, &conn)?;
    Ok(Some(()))
}

#[delete("/session/<session_id>")]
fn delete_session(session_id: String, keys: APIKey, pool: State<RedisPool>) -> Result<Option<()>> {
    let conn = pool.get().unwrap();

    let mut s = match session::Session::lookup(&session_id, &conn)? {
        Some(s) => s,
        None => return Ok(None),
    };
    if !is_admin(&keys, &s, &conn)? {
        bail!(ErrorKind::UserForbidden(
            "User is not authorized as session admin".to_owned()
        ))
    }
    s.delete(&conn)?;
    redisutil::update_session(&session_id, &conn)?;
    Ok(Some(()))
}

fn is_admin(keys: &APIKey, session: &session::Session, conn: &redis::Connection) -> Result<bool> {
    let authorized = if redisutil::check_token(&keys, &conn)? {
        let admins = session.get_associates("admin", &conn)?;
        // Make sure that the caller is one of the session admins
        if admins.contains(&keys.user_id) {
            true
        } else {
            false
        }
    } else {
        false
    };
    Ok(authorized)
}

pub fn lookup_active_users(session: &session::Session,
                           conn: &redis::Connection)
                           -> Result<Vec<user::User>> {
    let user_ids = session.get_associates("user", &conn)?;
    let user_ids_ref = user_ids.iter().map(|s| s.as_str()).collect();
    let possible_users = user::User::bulk_lookup(user_ids_ref, &conn)?;
    let users = possible_users.into_iter()
        .filter(|u| u.is_some())
        .map(|u| u.unwrap())
        .collect();
    Ok(users)
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
        .mount("/", routes![index,])
        .mount("/static", routes![files,])
        .mount("/api",
               routes![
            create_user,
            cast_vote,
            delete_user,
            create_session,
            join_session,
            kick_user,
            grant_admin,
            revoke_admin,
            update_session,
            lookup_session,
            delete_session,
        ])
        .manage(pool)
        .launch();
}
