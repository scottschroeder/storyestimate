use redis::{Commands, Connection, FromRedisValue, Value};
use super::errors::*;
use rustc_serialize::{Decodable, json};
use std::fmt::Debug;

const DEFAULT_REDIS_TTL: usize = 60 * 60 * 24; // 1 day

pub trait RedisBackend: Sized + Decodable + Debug {
    fn object_id(&self) -> String;
    fn object_name() -> String;

    fn object_ttl() -> Option<usize> {
        Some(DEFAULT_REDIS_TTL)
    }

    fn unique_key(&self) -> String {
        Self::redis_key(&self.object_id())
    }

    fn check_exists(id: &str, conn: &Connection) -> Result<bool> {
        let result: Value = conn.exists(Self::redis_key(id))
            .chain_err(|| "Error communicating with Redis")?;

        info!("Key Exists '{}': {:?}", Self::redis_key(id), result);
        match result {
            Value::Int(0) => Ok(false),
            Value::Int(1) => Ok(true),
            _ => bail!("Redis returned invalid type"),
        }
    }

    fn exists(&self, conn: &Connection) -> Result<bool> {
        Self::check_exists(&self.object_id(), conn)
    }

    fn redis_key(id: &str) -> String {
        format!("se_{}_{}", Self::object_name(), id)
    }

    fn lookup(id: &str, conn: &Connection) -> Result<Option<Self>> {
        Self::lookup_raw_key(&Self::redis_key(id), conn)
    }

    fn lookup_raw_key(redis_key: &str, conn: &Connection) -> Result<Option<Self>> {
        let value: Value = conn.get(redis_key)?;
        Self::deserialize(value)
    }

    // TODO: Fill this in once we have users to test with
    fn bulk_lookup(pattern: &str, conn: &Connection) -> Result<Vec<Self>> {
        let redis_key = Self::redis_key(pattern);
        let values: Vec<Value> = conn.keys(redis_key)?;
        let results: Vec<Self> = values.iter()
            .map(|ref v| {
                String::from_redis_value(&v)
                    .chain_err(|| "Could not parse string from KEYS command")
                    .and_then(|ref k| Self::lookup_raw_key(k, &conn))
                    .and_then(|o| o.ok_or(ErrorKind::RedisEmptyError(pattern.to_owned()).into()))
            })
            .collect::<Result<Vec<Self>>>()?;

        Ok(results)
    }

    fn delete(&mut self, conn: &Connection) -> Result<()> {
        match conn.del(self.unique_key())? {
            Value::Int(1) => Ok(()),
            err => {
                bail!("Redis was unable to delete {}, got '{:?}'",
                      self.unique_key(),
                      err)
            }
        }
    }

    fn deserialize(value: Value) -> Result<Option<Self>> {
        let redis_string: Option<String> = match value {
            Value::Nil => None,
            Value::Data(data) => {
                let s: String = String::from_redis_value(&Value::Data(data))?;
                Some(s)
            }
            _other => bail!("Unknown Redis Return Value: {:?}", _other),
        };
        if let Some(string_repr) = redis_string {
            Ok(Some(json::decode(&string_repr)
                    .chain_err(|| "Could not parse from string returned by redis")?
            ))
        } else {
            Ok(None)
        }
    }
}
