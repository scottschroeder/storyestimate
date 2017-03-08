use redis::{ToRedisArgs, FromRedisValue, RedisError, RedisResult, Value};
use rustc_serialize::json;

use errors::*;
use super::user::{PublicUser, User, VoteState};
use super::generator;
use super::redisutil::RedisBackend;


#[derive(RustcDecodable, RustcEncodable)]
#[derive(Debug)]
pub struct Session {
    pub session_id: String,
    pub session_admin_token: String,
    pub average: Option<f32>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, Copy)]
pub enum SessionState {
    Reset,
    Vote,
    Visible,
    Dirty,
}


#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct PublicSession {
    pub session_id: String,
    pub average: Option<f32>,
    pub users: Vec<PublicUser>,
    pub state: SessionState,

}

fn choose_session_state(users: &Vec<User>) -> SessionState {
    let mut state = SessionState::Reset;
    for user in users {
        match (state, user.vote) {
            (SessionState::Reset, VoteState::Empty) => state = SessionState::Vote,
            (SessionState::Reset, VoteState::Hidden(_)) => state = SessionState::Vote,
            (SessionState::Reset, VoteState::Visible(_)) => state = SessionState::Visible,
            (SessionState::Vote, VoteState::Visible(_)) => return SessionState::Dirty,
            (SessionState::Visible, VoteState::Empty) => return SessionState::Dirty,
            (SessionState::Visible, VoteState::Hidden(_)) => return SessionState::Dirty,
            _ => (),
        }
    }
    state
}

impl PublicSession {
    pub fn new(s: &Session, users: &Vec<User>) -> Self {
        PublicSession {
            session_id: s.session_id.clone(),
            average: s.average.clone(),
            state: choose_session_state(users),
            users: users.iter().map(|u| PublicUser::from(u)).collect(),
        }
    }
}

impl Session {
    pub fn new() -> Self {
        Session {
            session_id: generator::session_id(),
            session_admin_token: generator::authtoken(),
            average: None,
        }
    }

    pub fn take_votes(&mut self, users: &mut Vec<User>) {
        let mut total = 0u32;
        let mut count = 0u32;
        for user in users {
            user.reveal();
            match user.vote {
                VoteState::Visible(x) => {
                    total += x;
                    count += 1;
                }
                _ => (),
            }
        }
        self.average = Some(total as f32 / count as f32);
    }

    pub fn clear(&mut self, users: &mut Vec<User>) {
        for user in users.iter_mut() {
            user.clear();
        }
        self.average = None;
    }

    pub fn reset(&mut self, users: &mut Vec<User>) {
        for user in users.iter_mut() {
            user.reset();
        }
        self.average = None;
    }


    // pub fn kick_user(&mut self, nickname: &str) {
    //     users.retain(|ref x| x.nickname != nickname);
    // }
}

impl RedisBackend for Session {
    fn object_id(&self) -> String {
        self.session_id.clone()
    }

    fn object_name() -> String {
        "session".to_owned()
    }
}

//TODO: Should this just be copy, then we can drop the lifetime?
impl<'a> ToRedisArgs for &'a Session {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        vec![json::encode(&self).unwrap().into_bytes()]
    }
}

#[test]
fn create_session() {
    let s = Session::new();
    assert_eq!(s.average, None);
}

#[test]
fn unique_auth_token() {
    let s1 = Session::new();
    let s2 = Session::new();
    assert!(s1.session_admin_token != s2.session_admin_token);
}

#[test]
fn unique_session_id() {
    let s1 = Session::new();
    let s2 = Session::new();
    assert!(s1.session_id != s2.session_id);
}

#[test]
fn take_votes_average() {
    let mut s = Session::new();
    let mut u = User::new("1", "joe");
    let mut u2 = User::new("1", "john");
    u.vote(4);
    u2.vote(6);
    let mut users = vec![u, u2];
    s.take_votes(&mut users);
    assert_eq!(s.average, Some(5f32));
}
