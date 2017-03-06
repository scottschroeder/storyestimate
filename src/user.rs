use redis::{ToRedisArgs, FromRedisValue, RedisError, RedisResult, Value};
use rustc_serialize::json;

use errors::*;

#[derive(RustcDecodable, RustcEncodable)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VoteState {
    Empty,
    Hidden(u32),
    Visible(u32),
}

#[derive(RustcDecodable, RustcEncodable)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct User {
    pub user_token: u32,
    pub nickname: String,
    pub vote: VoteState
}

impl ToRedisArgs for User {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        vec![json::encode(&self).unwrap().into_bytes()]
    }
}

impl FromRedisValue for User {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let string_repr: String = String::from_redis_value(v)?;
        json::decode(&string_repr)
            .chain_err(|| "Could not parse from string returned by redis")
            .map_err(|e| RedisError::from(e))
    }
}

impl User {
    pub fn new(name: &str) -> Self {
        User {
            user_token: 1,
            nickname: name.to_owned(),
            vote: VoteState::Empty,
        }
    }

    pub fn vote(&mut self, points: u32) {
        self.vote = VoteState::Hidden(points);
    }

    /// Take any hidden votes and make them visible
    pub fn reveal(&mut self) {
        self.vote = match self.vote {
            VoteState::Empty => VoteState::Empty,
            VoteState::Hidden(x) => VoteState::Visible(x),
            VoteState::Visible(x) => VoteState::Visible(x),
        };
    }

    /// Take any visible votes and make them empty
    pub fn clear(&mut self) {
        self.vote = match self.vote {
            VoteState::Empty => VoteState::Empty,
            VoteState::Hidden(x) => VoteState::Hidden(x),
            VoteState::Visible(_) => VoteState::Empty,
        };
    }

    /// Take any votes and make them empty
    pub fn reset(&mut self) {
        self.vote = VoteState::Empty;
    }
}

#[test]
fn user_vote() {
    let mut u = User::new("joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    assert_eq!(u.vote, VoteState::Hidden(3));
}

#[test]
fn user_vote_reveal() {
    let mut u = User::new("joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    u.reveal();
    assert_eq!(u.vote, VoteState::Visible(3));
}

#[test]
fn user_vote_clear() {
    let mut u = User::new("joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    u.reveal();
    assert_eq!(u.vote, VoteState::Visible(3));
    u.clear();
    assert_eq!(u.vote, VoteState::Empty);
}

#[test]
fn user_dont_clear_hidden() {
    let mut u = User::new("joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    assert_eq!(u.vote, VoteState::Hidden(3));
    u.clear();
    assert_eq!(u.vote, VoteState::Hidden(3));
}

#[test]
fn user_vote_reset() {
    let mut u = User::new("joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    u.reset();
    assert_eq!(u.vote, VoteState::Empty);
}
