use redis::{ToRedisArgs};
use rustc_serialize::json;
use super::redisutil::RedisBackend;
use super::generator;

#[derive(RustcDecodable, RustcEncodable)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VoteState {
    Empty,
    Hidden(u32),
    Visible(u32),
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PublicVoteState {
    Empty,
    Hidden,
    Visible,
}

#[derive(RustcDecodable, RustcEncodable)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct User {
    pub user_token: String,
    pub session_id: String,
    pub nickname: String,
    pub vote: VoteState,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PublicUser {
    pub nickname: String,
    pub vote_state: PublicVoteState,
    pub vote_amount: Option<u32>
}

impl<'a> From<&'a User> for PublicUser {
    fn from(u: &User) -> PublicUser {
        let (vote_state, vote_amount) = match u.vote {
            VoteState::Empty => (PublicVoteState::Empty, None),
            VoteState::Hidden(_) => (PublicVoteState::Hidden, None),
            VoteState::Visible(x) => (PublicVoteState::Visible, Some(x)),
        };
        PublicUser{
            nickname: u.nickname.clone(),
            vote_state: vote_state,
            vote_amount: vote_amount,
        }
    }
}

impl<'a> ToRedisArgs for &'a User {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        vec![json::encode(&self).unwrap().into_bytes()]
    }
}


impl RedisBackend for User {
    fn object_id(&self) -> String {
        format!("{}_{}", self.session_id, self.nickname)
    }

    fn object_name() -> String {
        "user".to_owned()
    }
}

impl User {
    pub fn new(session_id: &str, name: &str) -> Self {
        User {
            user_token: generator::authtoken(),
            session_id: session_id.into(),
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
    let mut u = User::new("1", "joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    assert_eq!(u.vote, VoteState::Hidden(3));
}

#[test]
fn user_vote_reveal() {
    let mut u = User::new("1", "joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    u.reveal();
    assert_eq!(u.vote, VoteState::Visible(3));
}

#[test]
fn user_vote_clear() {
    let mut u = User::new("1", "joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    u.reveal();
    assert_eq!(u.vote, VoteState::Visible(3));
    u.clear();
    assert_eq!(u.vote, VoteState::Empty);
}

#[test]
fn user_dont_clear_hidden() {
    let mut u = User::new("1", "joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    assert_eq!(u.vote, VoteState::Hidden(3));
    u.clear();
    assert_eq!(u.vote, VoteState::Hidden(3));
}

#[test]
fn user_vote_reset() {
    let mut u = User::new("1", "joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    u.reset();
    assert_eq!(u.vote, VoteState::Empty);
}
