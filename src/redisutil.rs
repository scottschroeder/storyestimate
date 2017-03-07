use redis::{Commands, Connection, FromRedisValue, Value};
use super::errors::*;
use rustc_serialize::{Decodable, json};

pub trait RedisBackend: Sized + Decodable {
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

    fn lookup(id: String, conn: &Connection) -> Result<Option<Self>> {
        let redis_key = format!("se_{}_{}", Self::object_name(), id);
        let value: Value = conn.get(redis_key)?;
        Self::deserialize(value)
    }

    // TODO: Fill this in once we have users to test with
    fn bulk_lookup(pattern: String, conn: &Connection) -> Result<()>{
        let redis_key = format!("se_{}_{}", Self::object_name(), pattern);
        let value: Vec<Value> = conn.keys(redis_key)?;
        Ok(())
    }

    fn deserialize(value: Value) -> Result<Option<Self>> {
        debug!("Lookup Resulted in: {:?}", value);
        let redis_string: Option<String> = match value {
            Value::Nil => None,
            Value::Data(data) => {
                let s: String = String::from_redis_value(&Value::Data(data))?;
                Some(s)
            },
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
