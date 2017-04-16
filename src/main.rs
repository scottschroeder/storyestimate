#![feature(plugin)]
#![plugin(rocket_codegen)]
// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate redis;
extern crate rustc_serialize;
extern crate rand;
extern crate hyper;
extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use(log,info,debug,warn)]
extern crate log;
#[macro_use]
extern crate error_chain;
extern crate r2d2;
extern crate r2d2_redis;
extern crate num_cpus;

mod errors;
mod util;
mod user;
mod service;
mod dal;
mod estimates;
mod webapp;

use dal::{RedisDBManager, SharedMemoryDB};
use r2d2_redis::RedisConnectionManager;
use redis::{ConnectionAddr, ConnectionInfo};
use std::env;

#[cfg(not(feature = "redis_estimates"))]
fn get_backend() -> SharedMemoryDB {
    SharedMemoryDB::new()
}

#[cfg(feature = "redis_estimates")]
fn get_backend() -> RedisDBManager {
    let cpus = num_cpus::get() as u32;
    let redis_ctx = ConnectionInfo {
        addr: Box::new(ConnectionAddr::Tcp("127.0.0.1".to_owned(), 6379)),
        db: 0,
        passwd: None,
    };
    info!("Creating Redis Pool ({}x -> {:?})", cpus, redis_ctx);
    let config = r2d2::Config::builder().pool_size(cpus).build();
    let manager = RedisConnectionManager::new(redis_ctx).unwrap();
    let pool = r2d2::Pool::new(config, manager).unwrap();

    RedisDBManager::new(pool)
}

fn main() {
    let rocket = webapp::build_webapp(get_backend());
    rocket.launch();
}
