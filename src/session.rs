use redis::{ToRedisArgs, FromRedisValue, RedisError, RedisResult, Value};
use rustc_serialize::json;

use errors::*;
use super::user::{User, VoteState};

#[derive(RustcDecodable, RustcEncodable)]
#[derive(Debug)]
struct Session {
    session_id: u32,
    session_admin_token: u32,
    users: Vec<User>,
    average: Option<f32>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            session_id: 1,
            session_admin_token: 1,
            users: vec![],
            average: None,
        }
    }

    pub fn add_user(&mut self, user: User) {
        self.users.push(user);
    }

    pub fn take_votes(&mut self) {
        let mut total = 0u32;
        let mut count = 0u32;
        for user in &mut self.users {
            user.reveal();
            match user.vote {
                VoteState::Visible(x) => {
                    total += x;
                    count += 1;
                },
                _ => (),
            }
        }
        self.average = Some(total as f32 / count as f32);
    }

    pub fn clear(&mut self) {
        for user in &mut self.users {
            user.clear();
        }
        self.average = None;
    }

    pub fn reset(&mut self) {
        for user in &mut self.users {
            user.reset();
        }
        self.average = None;
    }

    pub fn kick_user(&mut self, nickname: &str) {
        self.users.retain(|ref x| x.nickname != nickname);
    }

}

impl ToRedisArgs for Session {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        vec![json::encode(&self).unwrap().into_bytes()]
    }
}

impl FromRedisValue for Session {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let string_repr: String = String::from_redis_value(v)?;
        json::decode(&string_repr)
            .chain_err(|| "Could not parse from string returned by redis")
            .map_err(|e| RedisError::from(e))
    }
}

#[test]
fn create_session() {
    let s = Session::new();
    assert_eq!(s.users, vec![]);
    assert_eq!(s.average, None);
}

#[test]
fn add_user_to_session() {
    let mut s = Session::new();
    assert_eq!(s.users, vec![]);
    let u = User::new("joe");
    let u_vec = vec![u.clone()];
    s.add_user(u);
    assert_eq!(u_vec, s.users);
}


#[test]
fn take_votes_average() {
    let mut s = Session::new();
    let mut u = User::new("joe");
    let mut u2 = User::new("john");
    u.vote(4);
    u2.vote(6);
    s.add_user(u);
    s.add_user(u2);
    s.take_votes();
    assert_eq!(s.average, Some(5f32));
}

#[test]
fn session_kick_user() {
    let mut s = Session::new();
    let u = User::new("joe");
    let u2 = User::new("john");
    let u_vec = vec![u.clone()];
    let u2_vec = vec![u.clone(), u2.clone()];
    s.add_user(u);
    s.add_user(u2);
    assert_eq!(s.users, u2_vec);
    s.kick_user("john");
    assert_eq!(s.users, u_vec);
}
