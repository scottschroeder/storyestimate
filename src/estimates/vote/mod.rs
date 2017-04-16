mod public;

pub use self::public::{PublicVote, PublicVoteState};

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VoteState {
    Empty,
    Hidden(u32),
    Visible(u32),
}

impl VoteState {
    pub fn new() -> Self {
        VoteState::Empty
    }

    /// Place a new vote
    pub fn vote(&mut self, amount: u32) {
        *self = VoteState::Hidden(amount);
    }

    /// Take any hidden votes and make them visible
    pub fn reveal(&mut self) {
        *self = match *self {
            VoteState::Empty => VoteState::Empty,
            VoteState::Hidden(x) => VoteState::Visible(x),
            VoteState::Visible(x) => VoteState::Visible(x),
        };
    }

    /// Take any visible votes and make them empty
    pub fn clear(&mut self) {
        *self = match *self {
            VoteState::Empty => VoteState::Empty,
            VoteState::Hidden(x) => VoteState::Hidden(x),
            VoteState::Visible(_) => VoteState::Empty,
        };
    }

    /// Take any votes and make them empty
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}


#[test]
fn place_vote() {
    let mut v = VoteState::new();
    assert_eq!(v, VoteState::Empty);
    v.vote(3);
    assert_eq!(v, VoteState::Hidden(3));
}

#[test]
fn vote_reveal() {
    let mut v = VoteState::new();
    assert_eq!(v, VoteState::Empty);
    v.vote(3);
    v.reveal();
    assert_eq!(v, VoteState::Visible(3));
}

#[test]
fn vote_clear() {
    let mut v = VoteState::new();
    assert_eq!(v, VoteState::Empty);
    v.vote(3);
    v.reveal();
    assert_eq!(v, VoteState::Visible(3));
    v.clear();
    assert_eq!(v, VoteState::Empty);
}

#[test]
fn dont_clear_hidden() {
    let mut v = VoteState::new();
    assert_eq!(v, VoteState::Empty);
    v.vote(3);
    assert_eq!(v, VoteState::Hidden(3));
    v.clear();
    assert_eq!(v, VoteState::Hidden(3));
}

#[test]
fn vote_reset() {
    let mut v = VoteState::new();
    assert_eq!(v, VoteState::Empty);
    v.vote(3);
    v.reset();
    assert_eq!(v, VoteState::Empty);
}
