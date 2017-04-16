use errors::*;
use r2d2;
use r2d2::PooledConnection;

use r2d2_redis;
use r2d2_redis::RedisConnectionManager;
use redis::{Commands, Connection, FromRedisValue, ToRedisArgs, Value};
use std::collections::BTreeMap;
type RedisPool = r2d2::Pool<r2d2_redis::RedisConnectionManager>;

pub trait FooData {
    fn get(&self, key: &str) -> Result<Option<String>>;
    fn set(&mut self, key: &str, value: &str) -> Result<()>;
}

pub struct MemoryFoo {
    data: BTreeMap<String, String>,
}

impl MemoryFoo {
    pub fn new() -> Self {
        MemoryFoo { data: BTreeMap::new() }
    }
}

impl FooData for MemoryFoo {
    fn get(&self, key: &str) -> Result<Option<String>> {
        Ok(self.data.get(key).map(|u| u.clone()))
    }
    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        self.data.insert(key.to_string(), value.to_string());
        Ok(())
    }
}

const REDIS_BASE_KEY: &str = "REDISFOO";

pub struct RedisFoo {
    conn: PooledConnection<r2d2_redis::RedisConnectionManager>,
}

impl RedisFoo {
    pub fn new(conn: PooledConnection<r2d2_redis::RedisConnectionManager>) -> Self {
        RedisFoo { conn: conn }
    }

    fn redis_key(id: &str) -> String {
        format!("{}_{}", REDIS_BASE_KEY, id)
    }
}

impl FooData for RedisFoo {
    fn get(&self, key: &str) -> Result<Option<String>> {
        let value: String = self.conn.get(RedisFoo::redis_key(key)).unwrap();
        Ok(Some(value))
    }
    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let _: Value = self.conn.set(RedisFoo::redis_key(key), value).unwrap();
        Ok(())
    }
}
