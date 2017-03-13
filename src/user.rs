use redis::ToRedisArgs;
use rustc_serialize::json;
use super::redisutil::RedisBackend;
use super::generator;
use super::APIKey;

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
    pub user_id: String,
    pub user_token: String,
    pub nickname: Option<String>,
    pub vote: VoteState,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PublicUser {
    pub user_id: String,
    pub nickname: Option<String>,
    pub is_admin: bool,
    pub vote_state: PublicVoteState,
    pub vote_amount: Option<u32>,
}

impl<'a> From<&'a User> for PublicUser {
    fn from(u: &User) -> PublicUser {
        let (vote_state, vote_amount) = match u.vote {
            VoteState::Empty => (PublicVoteState::Empty, None),
            VoteState::Hidden(_) => (PublicVoteState::Hidden, None),
            VoteState::Visible(x) => (PublicVoteState::Visible, Some(x)),
        };
        PublicUser {
            user_id: u.user_id.clone(),
            nickname: u.nickname.clone(),
            is_admin: false,
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
        self.user_id.clone()
    }

    fn object_name() -> String {
        "user".to_owned()
    }
}

impl User {
    pub fn new(name: Option<&str>) -> Self {
        User {
            user_id: generator::user_id(),
            user_token: generator::authtoken(),
            nickname: name.map(|s| s.to_owned()),
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

    pub fn is_authorized(&self, key: &APIKey) -> bool {
        if let Some(ref user_token) = key.user_key {
            self.user_id == key.user_id && self.user_token == *user_token
        } else {
            false
        }
    }
}

#[test]
fn user_vote() {
    let mut u = User::new(None);
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    assert_eq!(u.vote, VoteState::Hidden(3));
}

#[test]
fn user_vote_reveal() {
    let mut u = User::new(None);
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    u.reveal();
    assert_eq!(u.vote, VoteState::Visible(3));
}

#[test]
fn user_vote_clear() {
    let mut u = User::new(None);
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    u.reveal();
    assert_eq!(u.vote, VoteState::Visible(3));
    u.clear();
    assert_eq!(u.vote, VoteState::Empty);
}

#[test]
fn user_dont_clear_hidden() {
    let mut u = User::new(None);
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    assert_eq!(u.vote, VoteState::Hidden(3));
    u.clear();
    assert_eq!(u.vote, VoteState::Hidden(3));
}

#[test]
fn user_vote_reset() {
    let mut u = User::new(None);
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    u.reset();
    assert_eq!(u.vote, VoteState::Empty);
}
