use redis::{Commands, Connection, FromRedisValue, Value};
use super::errors::*;
use rustc_serialize::{Decodable, json};
use std::fmt::Debug;

pub trait RedisBackend: Sized + Decodable + Debug {
    fn object_id(&self) -> String;
    fn object_name() -> String;
    fn unique_key(&self) -> String {
        format!("se_{}_{}", Self::object_name(), self.object_id())
    }
    fn exists(&self, conn: &Connection) -> Result<bool> {
        let result: Value = conn.exists(self.unique_key())
            .chain_err(|| "Error communicating with Redis")?;

        info!("Key Exists '{}': {:?}", self.unique_key(), result);
        match result {
            Value::Int(0) => Ok(false),
            Value::Int(1) => Ok(true),
            _ => bail!("Redis returned invalid type"),
        }
    }

    fn lookup(id: &str, conn: &Connection) -> Result<Option<Self>> {
        let redis_key = format!("se_{}_{}", Self::object_name(), id);
        Self::lookup_raw_key(&redis_key, conn)
    }

    fn lookup_raw_key(redis_key: &str, conn: &Connection) -> Result<Option<Self>> {
        info!("Looking up: {}", redis_key);
        let value: Value = conn.get(redis_key)?;
        Self::deserialize(value)
    }

    // TODO: Fill this in once we have users to test with
    fn bulk_lookup(pattern: &str, conn: &Connection) -> Result<Vec<Self>> {
        let redis_key = format!("se_{}_{}", Self::object_name(), pattern);
        info!("redis-cli KEYS {}", redis_key);
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

    fn deserialize(value: Value) -> Result<Option<Self>> {
        info!("Lookup Resulted in: {:?}", value);
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
