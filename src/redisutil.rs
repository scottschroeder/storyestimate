use redis::{Commands, Connection, FromRedisValue, ToRedisArgs, Value};
use super::errors::*;
use rustc_serialize::{Decodable, json};
use std::fmt::Debug;
use super::APIKey;
use super::user;

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

    fn unique_associate_key(&self, relationship: &str) -> String {
        Self::redis_associate_key(&self.object_id(), relationship)
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

    fn redis_associate_key(id: &str, relationship: &str) -> String {
        format!("se_associate_{}_{}_{}",
                Self::object_name(),
                id,
                relationship)
    }

    fn lookup_strict(id: &str, conn: &Connection) -> Result<Self> {
        let redis_key = Self::redis_key(id);
        match Self::lookup_raw_key(&redis_key, conn)? {
            Some(x) => Ok(x),
            None => Err(ErrorKind::RedisEmptyError(format!("Could not find {}", redis_key)).into()),
        }
    }

    fn lookup(id: &str, conn: &Connection) -> Result<Option<Self>> {
        Self::lookup_raw_key(&Self::redis_key(id), conn)
    }

    fn lookup_raw_key(redis_key: &str, conn: &Connection) -> Result<Option<Self>> {
        let value: Value = conn.get(redis_key)?;
        Self::deserialize(value)
    }

    // TODO: Fill this in once we have users to test with

    fn pattern_lookup(pattern: &str, conn: &Connection) -> Result<Vec<Self>> {
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

    fn bulk_lookup(ids: Vec<&str>, conn: &Connection) -> Result<Vec<Option<Self>>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let redis_keys: Vec<String> = ids.iter().map(|user_id| Self::redis_key(user_id)).collect();
        let values: Vec<Value> = conn.get(redis_keys)?;
        let results: Vec<Option<Self>> = values.into_iter()
            .map(|rv| Self::deserialize(rv))
            .collect::<Result<_>>()?;
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


    fn associate(&self, foreign: &str, relationship: &str, conn: &Connection) -> Result<()> {
        let redis_key = self.unique_associate_key(relationship);
        let result: Value = conn.sadd(&redis_key, foreign)?;
        match result {
            Value::Int(_) => Ok(()),
            _ => {
                bail!("Redis unable to publish, and did not report error: {:?}",
                      result)
            }
        }
    }

    fn get_associates(&self, relationship: &str, conn: &Connection) -> Result<Vec<String>> {
        let redis_key = self.unique_associate_key(relationship);
        let result: Value = conn.smembers(&redis_key)?;
        info!("Associates({}) [{}]: {:?}",
              self.object_id(),
              relationship,
              result);
        match result {
            Value::Bulk(value_vec) => Ok(String::from_redis_values(value_vec.as_slice())?),
            _ => bail!("Redis did not return expected set of strings: {:?}", result),
        }
    }

    fn disassociate(&self, foreign: &str, relationship: &str, conn: &Connection) -> Result<()> {
        let redis_key = self.unique_associate_key(relationship);
        let result: Value = conn.srem(&redis_key, foreign)?;
        match result {
            Value::Int(_) => Ok(()),
            _ => {
                bail!("Redis unable to publish, and did not report error: {:?}",
                      result)
            }
        }
    }
}

// This magic is "Higher Rank Trait Bounds": https://doc.rust-lang.org/nomicon/hrtb.html

pub fn save<T>(obj: &T, conn: &Connection) -> Result<()>
    where T: RedisBackend,
          for<'a> &'a T: ToRedisArgs
{

    let result: Value = match T::object_ttl() {
        Some(ttl) => conn.set_ex(obj.unique_key(), obj, ttl),
        None => conn.set(obj.unique_key(), obj),
    }?;
    match result {
        Value::Okay => Ok(()),
        _ => bail!("Tried to save {:?}, got result: {:?}", obj, result),

    }
}

pub fn check_token(key: &APIKey, conn: &Connection) -> Result<bool> {
    match user::User::lookup(&key.user_id, &conn)? {
        Some(u) => Ok(u.is_authorized(key)),
        None => Ok(false),
    }
}


fn publish(channel: &str, msg: &str, conn: &Connection) -> Result<()> {
    let result = conn.publish(channel, msg)?;
    match result {
        Value::Int(_) => Ok(()),
        _ => {
            bail!("Redis unable to publish, and did not report error: {:?}",
                  result)
        }
    }
}

pub fn update_session(session_id: &str, conn: &Connection) -> Result<()> {
    publish("se_session_update", session_id, conn)
}
