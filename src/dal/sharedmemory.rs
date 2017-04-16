use super::{MemoryDB, RedisDB, StoryDataProvider};
use r2d2;
use r2d2_redis;
use std::sync::{Mutex, MutexGuard};

pub struct SharedMemoryDB {
    inner: Mutex<MemoryDB>,
}


impl SharedMemoryDB {
    pub fn new() -> Self {
        SharedMemoryDB { inner: Mutex::new(MemoryDB::new()) }
    }
    pub fn get(&self) -> MutexGuard<MemoryDB> {
        self.inner.lock().unwrap()
    }
}

impl StoryDataProvider for SharedMemoryDB {}
//     type D = MemoryDB;

//     fn get_story_data<'a>(&'a mut self) -> &'a mut Self::D {
//         self.inner.lock().unwrap()
//     }
// }

type RedisPool = r2d2::Pool<r2d2_redis::RedisConnectionManager>;

pub struct RedisDBManager {
    inner: RedisPool,
}

pub struct RedisDBInstance(RedisDB);


impl RedisDBManager {
    pub fn new(pool: RedisPool) -> Self {
        RedisDBManager { inner: pool }
    }
    pub fn get(&self) -> RedisDBInstance {
        let conn = self.inner.get().unwrap();
        RedisDBInstance(RedisDB::new(conn))
    }
}

impl StoryDataProvider for RedisDBManager {}
use std::ops::{Deref, DerefMut};

impl Deref for RedisDBInstance {
    type Target = RedisDB;

    fn deref(&self) -> &RedisDB {
        &self.0
    }
}

impl DerefMut for RedisDBInstance {
    fn deref_mut(&mut self) -> &mut RedisDB {
        &mut self.0
    }
}
